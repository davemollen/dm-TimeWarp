use grain_stretch::{GrainStretch, Notes, Params, WavProcessor};
use lv2::prelude::*;
use std::string::String;
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
  time: InputPort<Control>,
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
  control: InputPort<AtomPort>,
  input_left: InputPort<Audio>,
  input_right: InputPort<Audio>,
  output_left: OutputPort<Audio>,
  output_right: OutputPort<Audio>,
}

#[derive(FeatureCollection)]
pub struct Features<'a> {
  map: LV2Map<'a>,
}

#[derive(URIDCollection)]
pub struct URIDs {
  atom: AtomURIDCollection,
  midi: MidiURIDCollection,
  unit: UnitURIDCollection,
}

#[uri("https://github.com/davemollen/dm-GrainStretch")]
struct DmGrainStretch {
  grain_stretch: GrainStretch,
  params: Params,
  urids: URIDs,
  notes: Notes,
  wav_processor: WavProcessor,
  loaded_file_path: Option<String>,
}

impl State for DmGrainStretch {
  type StateFeatures = Features<'static>;

  fn save(&self, mut store: StoreHandle, features: Self::StateFeatures) -> Result<(), StateErr> {
    let path = self.loaded_file_path.as_ref().ok_or(StateErr::Unknown)?;
    let property_key = features.map.map_str("sample").ok_or(StateErr::Unknown)?;
    let mut state_property_writer = store.draft(property_key);
    state_property_writer
      .init(self.urids.atom.string)?
      .append(path)
      .or(Err(StateErr::Unknown))?;
    store.commit(property_key);

    Ok(())
  }

  fn restore(
    &mut self,
    store: RetrieveHandle,
    features: Self::StateFeatures,
  ) -> Result<(), StateErr> {
    let property_key = features.map.map_str("sample").ok_or(StateErr::Unknown)?;
    let reader = store.retrieve(property_key)?;
    let path = reader.read(self.urids.atom.string)?;
    self.loaded_file_path = Some(path.to_string());

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

  pub fn process_audio_file(&mut self, ports: &mut Ports) {
    let control_sequence = match ports
      .control
      .read(self.urids.atom.sequence)
      .and_then(|s| s.with_unit(self.urids.unit.frame))
    {
      Ok(sequence_iter) => sequence_iter,
      Err(_) => return,
    };

    for (_, atom) in control_sequence {
      let path: String = match atom.read(self.urids.atom.string) {
        Ok(path) => path.to_string(),
        Err(_) => continue,
      };
      self.load_wav_file(path);
    }
  }

  fn load_wav_file(&mut self, path: String) {
    if path.is_empty() || self.loaded_file_path.as_ref().is_some_and(|x| *x == path) {
      return;
    }
    match self.wav_processor.read_wav(&path) {
      Ok(samples) => {
        self.grain_stretch.load_wav_file(samples);
      }
      Err(_) => (),
    };
    self.loaded_file_path = Some(path);
  }
}

impl Plugin for DmGrainStretch {
  // Tell the framework which ports this plugin has.
  type Ports = Ports;

  // We don't need any special host features; We can leave them out.
  type InitFeatures = Features<'static>;
  type AudioFeatures = ();

  // Create a new instance of the plugin; Trivial in this case.
  fn new(plugin_info: &PluginInfo, features: &mut Features<'static>) -> Option<Self> {
    let sample_rate = plugin_info.sample_rate() as f32;

    Some(Self {
      grain_stretch: GrainStretch::new(sample_rate),
      params: Params::new(sample_rate),
      urids: features.map.populate_collection()?,
      notes: Notes::new(),
      wav_processor: WavProcessor::new(sample_rate),
      loaded_file_path: None,
    })
  }

  // Process a chunk of audio. The audio ports are dereferenced to slices, which the plugin
  // iterates over.
  fn run(&mut self, ports: &mut Ports, _features: &mut (), _sample_count: u32) {
    self.params.set(
      *ports.scan,
      *ports.spray,
      *ports.size,
      *ports.speed,
      *ports.density,
      *ports.stretch,
      *ports.record,
      *ports.time,
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
    );

    self.process_midi_events(ports);
    self.process_audio_file(ports);

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
}

// Generate the plugin descriptor function which exports the plugin to the outside world.
lv2_descriptors!(DmGrainStretch);
