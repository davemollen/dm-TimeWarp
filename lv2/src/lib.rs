mod state;
mod worker;
use lv2::prelude::*;
use std::string::String;
use time_warp::{Notes, Params, TimeMode, TimeWarp};
use wmidi::*;
use worker::*;

#[derive(PortCollection)]
struct Ports {
  scan: InputPort<Control>,
  spray: InputPort<Control>,
  size: InputPort<Control>,
  speed: InputPort<Control>,
  density: InputPort<Control>,
  stretch: InputPort<Control>,
  record: InputPort<Control>,
  play: InputPort<Control>,
  time_mode: InputPort<Control>,
  time: InputPort<Control>,
  time_multiply: InputPort<Control>,
  highpass: InputPort<Control>,
  lowpass: InputPort<Control>,
  feedback: InputPort<Control>,
  recycle: InputPort<Control>,
  dry: InputPort<Control>,
  wet: InputPort<Control>,
  midi_enabled: InputPort<Control>,
  voices: InputPort<Control>,
  attack: InputPort<Control>,
  decay: InputPort<Control>,
  sustain: InputPort<Control>,
  release: InputPort<Control>,
  clear: InputPort<Control>,
  control: InputPort<AtomPort>,
  input_left: InputPort<Audio>,
  input_right: InputPort<Audio>,
  output_left: OutputPort<Audio>,
  output_right: OutputPort<Audio>,
}

#[derive(FeatureCollection)]
struct InitFeatures<'a> {
  map: LV2Map<'a>,
}

#[derive(FeatureCollection)]
struct AudioFeatures<'a> {
  schedule: Schedule<'a, DmTimeWarp>,
}

#[uri("https://github.com/davemollen/dm-TimeWarp#sample")]
struct Sample;

#[derive(URIDCollection)]
struct URIDs {
  atom: AtomURIDCollection,
  midi: MidiURIDCollection,
  unit: UnitURIDCollection,
  patch: PatchURIDCollection,
  log: LogURIDCollection,
  sample: URID<Sample>,
}

#[uri("https://github.com/davemollen/dm-TimeWarp")]
struct DmTimeWarp {
  time_warp: TimeWarp,
  params: Params,
  urids: URIDs,
  notes: Notes,
  activated: bool,
  worker_is_initialized: bool,
  file_path: String,
  sample_rate: f32,
}

impl DmTimeWarp {
  pub fn set_param_values(&mut self, ports: &mut Ports, sample_count: u32) {
    self.params.set(
      *ports.scan,
      *ports.spray,
      *ports.size,
      *ports.speed,
      *ports.density,
      *ports.stretch,
      *ports.record == 1.,
      *ports.play == 1.,
      match *ports.time_mode {
        1. => TimeMode::Delay,
        _ => TimeMode::Looper,
      },
      *ports.time,
      *ports.time_multiply,
      *ports.highpass,
      *ports.lowpass,
      *ports.feedback,
      *ports.recycle,
      *ports.dry,
      *ports.wet,
      *ports.midi_enabled == 1.,
      *ports.attack,
      *ports.decay,
      *ports.sustain,
      *ports.release,
      *ports.clear == 1.,
      self.time_warp.get_delay_line(),
      sample_count as usize,
    );

    if self.params.should_clear_buffer() {
      self.file_path = "".to_string();
    }
  }

  // TODO: see if we can process patch and midi event in one loop instead
  pub fn process_midi_events(&mut self, ports: &mut Ports) {
    let control_sequence = match ports
      .control
      .read(self.urids.atom.sequence)
      .and_then(|s| s.with_unit(self.urids.unit.frame))
    {
      Ok(sequence_iter) => sequence_iter,
      Err(_) => return,
    };

    for (_, atom) in control_sequence {
      let midi_message = match atom.read(self.urids.midi.wmidi) {
        Ok(message) => message,
        _ => continue,
      };

      match midi_message {
        MidiMessage::NoteOn(_, note, velocity) => {
          self
            .notes
            .note_on(note.into(), (u8::from(velocity) / 127).into());
        }
        MidiMessage::NoteOff(_, note, _) => {
          self.notes.note_off(note.into());
        }
        _ => (),
      }
    }

    self.notes.set_voice_count(*ports.voices as usize);
  }

  pub fn process_patch_events(&mut self, ports: &mut Ports, features: &mut AudioFeatures) {
    let control_sequence = match ports
      .control
      .read(self.urids.atom.sequence)
      .and_then(|s| s.with_unit(self.urids.unit.frame))
    {
      Ok(sequence_iter) => sequence_iter,
      Err(_) => return,
    };

    let mut should_read_patch_value = false;
    for (_, atom) in control_sequence {
      let (object_header, object_reader) = match atom
        .read(self.urids.atom.object)
        .or_else(|_| atom.read(self.urids.atom.blank))
      {
        Ok(x) => x,
        Err(_) => {
          continue;
        }
      };

      if object_header.otype == self.urids.patch.set_class {
        for (property_header, property) in object_reader {
          if property_header.key == self.urids.patch.property {
            match property.read(self.urids.atom.urid) {
              Ok(patch_property) => {
                should_read_patch_value = self.urids.sample.get() == patch_property.get()
              }
              Err(_) => continue,
            }
          }
          if should_read_patch_value && property_header.key == self.urids.patch.value {
            self.file_path = match property.read(self.urids.atom.path) {
              Ok(f) => f.to_string(),
              Err(_) => continue,
            };
            features
              .schedule
              .schedule_work(WorkData::new(&self.file_path, self.sample_rate))
              .ok();
          }
        }
      }
    }
  }
}

impl Plugin for DmTimeWarp {
  type Ports = Ports;
  type InitFeatures = InitFeatures<'static>;
  type AudioFeatures = AudioFeatures<'static>;

  fn new(plugin_info: &PluginInfo, features: &mut Self::InitFeatures) -> Option<Self> {
    let sample_rate = plugin_info.sample_rate() as f32;

    Some(Self {
      time_warp: TimeWarp::new(sample_rate),
      params: Params::new(sample_rate),
      urids: features.map.populate_collection()?,
      notes: Notes::new(),
      activated: false,
      worker_is_initialized: true,
      file_path: "".to_string(),
      sample_rate,
    })
  }

  fn run(&mut self, ports: &mut Ports, features: &mut Self::AudioFeatures, sample_count: u32) {
    if self.activated && self.worker_is_initialized {
      if !self.file_path.is_empty() {
        features
          .schedule
          .schedule_work(WorkData::new(&self.file_path, self.sample_rate))
          .ok();
      }
      self.worker_is_initialized = false;
    }
    self.set_param_values(ports, sample_count);
    self.process_midi_events(ports);
    self.process_patch_events(ports, features);

    let input_channels = ports.input_left.iter().zip(ports.input_right.iter());
    let output_channels = ports
      .output_left
      .iter_mut()
      .zip(ports.output_right.iter_mut());

    for ((input_left, input_right), (output_left, output_right)) in
      input_channels.zip(output_channels)
    {
      (*output_left, *output_right) = self.time_warp.process(
        (*input_left, *input_right),
        &mut self.params,
        &mut self.notes.get_notes(),
      );
    }
  }

  fn extension_data(uri: &Uri) -> Option<&'static dyn std::any::Any> {
    match_extensions!(uri, StateDescriptor<Self>, WorkerDescriptor<Self>)
  }

  fn activate(&mut self, _features: &mut Self::InitFeatures) {
    self.activated = true;
  }

  fn deactivate(&mut self, _features: &mut Self::InitFeatures) {
    self.activated = false;
  }
}

// Generate the plugin descriptor function which exports the plugin to the outside world.
lv2_descriptors!(DmTimeWarp);
