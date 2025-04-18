use grain_stretch::{GrainStretch, Notes, Params, TimeMode};
use lv2::prelude::{
  path::{FreePath, MakePath, MapPath, PathManager},
  *,
};
use std::{path::Path, string::String};
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
pub struct Features<'a> {
  map: LV2Map<'a>,
  make_path: Option<MakePath<'a>>,
  map_path: Option<MapPath<'a>>,
  free_path: Option<FreePath<'a>>,
}

#[uri("https://github.com/davemollen/dm-GrainStretch#sample")]
struct Sample;

#[derive(URIDCollection)]
pub struct URIDs {
  atom: AtomURIDCollection,
  midi: MidiURIDCollection,
  unit: UnitURIDCollection,
  patch: PatchURIDCollection,
  sample: URID<Sample>,
}

#[uri("https://github.com/davemollen/dm-GrainStretch")]
struct DmGrainStretch {
  grain_stretch: GrainStretch,
  params: Params,
  urids: URIDs,
  notes: Notes,
  activated: bool,
}

impl State for DmGrainStretch {
  type StateFeatures = Features<'static>;

  fn save(&self, mut store: StoreHandle, features: Self::StateFeatures) -> Result<(), StateErr> {
    match (features.make_path, features.map_path, features.free_path) {
      (Some(make_path), Some(map_path), Some(free_path)) => {
        let mut manager = PathManager::new(make_path, map_path, free_path);

        let (_, abstract_path) = manager.allocate_path(Path::new(&self.params.file_path))?;

        let _ = store
          .draft(self.urids.sample)
          .init(self.urids.atom.path)?
          .append(&*abstract_path);

        store.commit_all()
      }
      (_, _, _) => Ok(()),
    }
  }
  fn restore(
    &mut self,
    store: RetrieveHandle,
    features: Self::StateFeatures,
  ) -> Result<(), StateErr> {
    match (
      features.make_path,
      features.map_path,
      features.free_path,
      self.activated,
    ) {
      (Some(make_path), Some(map_path), Some(free_path), true) => {
        let mut manager = PathManager::new(make_path, map_path, free_path);

        let abstract_path = store
          .retrieve(self.urids.sample)?
          .read(self.urids.atom.path)?;

        self.params.file_path = manager
          .deabstract_path(abstract_path)?
          .to_string_lossy()
          .to_string();

        Ok(())
      }
      (_, _, _, _) => Ok(()),
    }
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
            self.params.file_path = match property.read(self.urids.atom.path) {
              Ok(f) => f.to_string(),
              Err(_) => continue,
            };
          }
        }
      }
    }
  }
}

impl Plugin for DmGrainStretch {
  // Tell the framework which ports this plugin has.
  type Ports = Ports;

  // We don't need any special host features; We can leave them out.
  type InitFeatures = Features<'static>;
  type AudioFeatures = ();

  // Create a new instance of the plugin; Trivial in this case.
  fn new(plugin_info: &PluginInfo, features: &mut Self::InitFeatures) -> Option<Self> {
    let sample_rate = plugin_info.sample_rate() as f32;

    Some(Self {
      grain_stretch: GrainStretch::new(sample_rate),
      params: Params::new(sample_rate),
      urids: features.map.populate_collection()?,
      notes: Notes::new(),
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
      None,
      *ports.clear == 1.,
      self.grain_stretch.get_delay_line(),
    );
    self.process_midi_events(ports);
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
