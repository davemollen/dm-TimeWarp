use grain_stretch::{GrainStretch, Notes, Params, TimeMode, WavProcessor};
use lv2::prelude::*;
use std::{ffi::CStr, string::String};
use wmidi::*;

#[derive(PortCollection)]
struct Ports {
  scan: InputPort<Control>,
  spray: InputPort<Control>,
  size: InputPort<Control>,
  speed: InputPort<Control>,
  density: InputPort<Control>,
  stretch: InputPort<Control>,
  record: InputPort<Control>,
  time_mode: InputPort<Control>,
  time: InputPort<Control>,
  time_multiply: InputPort<Control>,
  highpass: InputPort<Control>,
  lowpass: InputPort<Control>,
  overdub: InputPort<Control>,
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
pub struct Features<'a> {
  map: LV2Map<'a>,
  unmap: LV2Unmap<'a>,
  log: Log<'a>,
}

#[derive(FeatureCollection)]
pub struct AudioFeatures<'a> {
  log: Log<'a>,
}

#[uri("https://github.com/davemollen/dm-GrainStretch#sample")]
struct Sample;

#[derive(URIDCollection)]
pub struct URIDs {
  atom: AtomURIDCollection,
  midi: MidiURIDCollection,
  unit: UnitURIDCollection,
  patch: PatchURIDCollection,
  log: LogURIDCollection,
  sample: URID<Sample>,
}

#[uri("https://github.com/davemollen/dm-GrainStretch")]
struct DmGrainStretch {
  grain_stretch: GrainStretch,
  params: Params,
  urids: URIDs,
  notes: Notes,
  wav_processor: WavProcessor,
  file_path: String,
  loaded_file_path: Option<String>,
  file_duration: Option<f32>,
  activated: bool,
}

impl State for DmGrainStretch {
  type StateFeatures = Features<'static>;

  fn save(&self, mut store: StoreHandle, features: Self::StateFeatures) -> Result<(), StateErr> {
    let message = "saving state: {}\n\0".to_string();
    let _ = features.log.print_cstr(
      self.urids.log.note,
      CStr::from_bytes_with_nul(message.as_bytes()).unwrap(),
    );
    Ok(())
  }

  fn restore(
    &mut self,
    store: RetrieveHandle,
    _features: Self::StateFeatures,
  ) -> Result<(), StateErr> {
    if !self.activated {
      let property = store.retrieve(self.urids.sample)?;
      let path = property.read(self.urids.atom.path)?.to_string();
      self.file_path = path;
    }

    Ok(())
  }
}

impl DmGrainStretch {
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

  pub fn process_patch_events(&mut self, ports: &mut Ports) {
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
          }
        }
      }
    }
  }

  fn process_audio_file(&mut self, ports: &mut Ports) {
    if *ports.clear == 1. {
      self.grain_stretch.clear_buffer();
      self.loaded_file_path = None;
      self.file_path = "".to_string();
      self.file_duration = None;
      return;
    }

    if self.file_path.is_empty()
      || self
        .loaded_file_path
        .as_ref()
        .is_some_and(|x| *x == self.file_path)
    {
      return;
    }
    if let Ok(samples) = self.wav_processor.read_wav(&self.file_path) {
      self.grain_stretch.set_buffer(samples);
    };
    if let Ok(duration) = self.wav_processor.get_duration(&self.file_path) {
      self.file_duration = Some(duration);
    };
    self.loaded_file_path = Some(self.file_path.clone());
  }
}

impl Plugin for DmGrainStretch {
  // Tell the framework which ports this plugin has.
  type Ports = Ports;

  // We don't need any special host features; We can leave them out.
  type InitFeatures = Features<'static>;
  type AudioFeatures = AudioFeatures<'static>;

  // Create a new instance of the plugin; Trivial in this case.
  fn new(plugin_info: &PluginInfo, features: &mut Self::InitFeatures) -> Option<Self> {
    let sample_rate = plugin_info.sample_rate() as f32;

    Some(Self {
      grain_stretch: GrainStretch::new(sample_rate),
      params: Params::new(sample_rate),
      urids: features.map.populate_collection()?,
      notes: Notes::new(),
      wav_processor: WavProcessor::new(sample_rate),
      file_path: "".to_string(),
      loaded_file_path: None,
      file_duration: None,
      activated: false,
    })
  }

  // Process a chunk of audio. The audio ports are dereferenced to slices, which the plugin
  // iterates over.
  fn run(&mut self, ports: &mut Ports, _features: &mut Self::AudioFeatures, _sample_count: u32) {
    self.params.set(
      *ports.scan,
      *ports.spray,
      *ports.size,
      *ports.speed,
      *ports.density,
      *ports.stretch,
      *ports.record == 1.,
      match *ports.time_mode {
        1. => TimeMode::Delay,
        _ => TimeMode::Looper,
      },
      *ports.time,
      *ports.time_multiply,
      *ports.highpass,
      *ports.lowpass,
      *ports.overdub,
      *ports.recycle,
      *ports.dry,
      *ports.wet,
      *ports.midi_enabled == 1.,
      *ports.attack,
      *ports.decay,
      *ports.sustain,
      *ports.release,
      self.file_duration,
      *ports.clear == 1.,
    );
    self.process_midi_events(ports);
    self.process_audio_file(ports);
    self.process_patch_events(ports);

    let input_channels = ports.input_left.iter().zip(ports.input_right.iter());
    let output_channels = ports
      .output_left
      .iter_mut()
      .zip(ports.output_right.iter_mut());

    for ((input_left, input_right), (output_left, output_right)) in
      input_channels.zip(output_channels)
    {
      (*output_left, *output_right) = self.grain_stretch.process(
        (*input_left, *input_right),
        &mut self.params,
        &mut self.notes.get_notes(),
      );
    }
  }

  fn extension_data(uri: &Uri) -> Option<&'static dyn std::any::Any> {
    match_extensions!(uri, StateDescriptor<Self>)
  }

  fn activate(&mut self, _features: &mut Self::InitFeatures) {
    self.activated = true;
  }

  fn deactivate(&mut self, _features: &mut Self::InitFeatures) {
    self.activated = false;
  }
}

// Generate the plugin descriptor function which exports the plugin to the outside world.
lv2_descriptors!(DmGrainStretch);
