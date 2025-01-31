extern crate grain_stretch;
extern crate lv2;
use std::collections::HashMap;
use grain_stretch::{GrainStretch, Params, Note};
use lv2::prelude::*;
use wmidi::MidiMessage;

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
  input_left: InputPort<Audio>,
  input_right: InputPort<Audio>,
  output_left: OutputPort<Audio>,
  output_right: OutputPort<Audio>,
  control: InputPort<AtomPort>
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
  notes: HashMap<u8, Note>
}

impl DmGrainStretch {
  pub fn process_midi_events(&mut self, ports: &mut Ports) {
    let sequence_header_reader = match ports.control.read(self.urids.atom.sequence) {
      Ok(sequence_header_reader) => sequence_header_reader,
      Err(_) => return,
    };
    let sequence_iter = match sequence_header_reader.with_unit(self.urids.unit.beat) {
      Ok(sequence_iter) => sequence_iter,
      Err(_) => return,
    };  
      
    for (_, message) in sequence_iter {
      let midi_message = match message.read(self.urids.midi.wmidi) {
        Ok(midi_message) => midi_message,
        Err(_) => return,
      };

      match midi_message {
        MidiMessage::NoteOn(_, note, velocity) => {
          let note: u8 = note.into();
          let velocity: f32 =  (u8::from(velocity) / 127).into();
          self.notes.insert(note, Note::new(note, velocity));
        },
        MidiMessage::NoteOff(_, note, _) => {
          let note: u8 = note.into();
          if self.notes.contains_key(&note) {
            self.notes.remove(&note);
          }
        },
        _ => (),
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
  fn new(plugin_info: &PluginInfo, features: &mut Features<'static>) -> Option<Self> {
    let sample_rate = plugin_info.sample_rate() as f32;

    Some(Self {
      grain_stretch: GrainStretch::new(sample_rate),
      params: Params::new(sample_rate),
      urids: features.map.populate_collection()?,
      notes: HashMap::new(),
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
    );
    self.process_midi_events(ports);

    let input_channels = ports.input_left.iter().zip(ports.input_right.iter());
    let output_channels = ports
      .output_left
      .iter_mut()
      .zip(ports.output_right.iter_mut());

    for ((input_left, input_right), (output_left, output_right)) in
      input_channels.zip(output_channels)
    {
      (*output_left, *output_right) = self
        .grain_stretch
        .process((*input_left, *input_right), &mut self.params, &self.notes);
    }
  }
}

// Generate the plugin descriptor function which exports the plugin to the outside world.
lv2_descriptors!(DmGrainStretch);
