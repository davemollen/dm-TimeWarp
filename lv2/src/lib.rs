mod events;
mod state;
mod worker;
use lv2::prelude::*;
use std::string::String;
use time_warp::{Notes, Params, SampleMode, TimeWarp};
use worker::*;

#[derive(PortCollection)]
struct Ports {
  scan: InputPort<InPlaceControl>,
  spray: InputPort<InPlaceControl>,
  size: InputPort<InPlaceControl>,
  density: InputPort<InPlaceControl>,
  stereo: InputPort<InPlaceControl>,
  pitch: InputPort<InPlaceControl>,
  stretch: InputPort<InPlaceControl>,
  record: InputPort<InPlaceControl>,
  play: InputPort<InPlaceControl>,
  sample_mode: InputPort<InPlaceControl>,
  time: InputPort<InPlaceControl>,
  length: InputPort<InPlaceControl>,
  highpass: InputPort<InPlaceControl>,
  lowpass: InputPort<InPlaceControl>,
  feedback: InputPort<InPlaceControl>,
  recycle: InputPort<InPlaceControl>,
  dry: InputPort<InPlaceControl>,
  wet: InputPort<InPlaceControl>,
  midi_enabled: InputPort<InPlaceControl>,
  voices: InputPort<InPlaceControl>,
  attack: InputPort<InPlaceControl>,
  decay: InputPort<InPlaceControl>,
  sustain: InputPort<InPlaceControl>,
  release: InputPort<InPlaceControl>,
  erase: InputPort<InPlaceControl>,
  control: InputPort<AtomPort>,
  notify: OutputPort<AtomPort>,
  input_left: InputPort<InPlaceAudio>,
  input_right: InputPort<InPlaceAudio>,
  output_left: OutputPort<InPlaceAudio>,
  output_right: OutputPort<InPlaceAudio>,
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
  worker_is_finished: bool,
  file_path: String,
  time_stamp: i64,
  sample_rate: f32,
}

impl DmTimeWarp {
  pub fn set_param_values(
    &mut self,
    ports: &mut Ports,
    features: &mut AudioFeatures,
    sample_count: u32,
  ) {
    self.params.set(
      ports.scan.get(),
      ports.spray.get(),
      ports.size.get(),
      ports.density.get(),
      ports.stereo.get(),
      ports.pitch.get(),
      ports.stretch.get(),
      ports.record.get() == 1.,
      ports.play.get() == 1.,
      match ports.sample_mode.get() {
        1. => SampleMode::Delay,
        2. => SampleMode::Looper,
        _ => SampleMode::Sampler,
      },
      ports.time.get(),
      ports.length.get(),
      ports.recycle.get(),
      ports.feedback.get(),
      ports.dry.get(),
      ports.wet.get(),
      ports.midi_enabled.get() == 1.,
      ports.attack.get(),
      ports.decay.get(),
      ports.sustain.get(),
      ports.release.get(),
      ports.erase.get() == 1.,
      sample_count as usize,
    );

    self
      .time_warp
      .get_filter()
      .set_coefficients(ports.highpass.get(), ports.lowpass.get());

    if self.params.should_erase_buffer() {
      self.file_path = "".to_string();
      self.write_set_file(ports);
      features
        .schedule
        .schedule_work(WorkRequest::FlushBuffer(
          self.time_warp.get_delay_line_size(),
        ))
        .ok();
    }

    self.notes.set_voice_count(ports.voices.get() as usize);
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
      worker_is_initialized: false,
      worker_is_finished: false,
      file_path: "".to_string(),
      time_stamp: 0,
      sample_rate,
    })
  }

  fn run(&mut self, ports: &mut Ports, features: &mut Self::AudioFeatures, sample_count: u32) {
    self.set_param_values(ports, features, sample_count);
    self.handle_events(ports, features);

    if self.activated && !self.worker_is_initialized {
      if !self.file_path.is_empty() {
        features
          .schedule
          .schedule_work(WorkRequest::LoadFile(
            self.file_path.to_string(),
            self.sample_rate,
            self.time_warp.get_delay_line_size(),
          ))
          .ok();
      }
      self.worker_is_initialized = true;
    }

    if self.worker_is_finished {
      self.write_set_file(ports);
      self.worker_is_finished = false;
    }

    let input_channels = ports.input_left.iter().zip(ports.input_right.iter());
    let output_channels = ports.output_left.iter().zip(ports.output_right.iter());

    for ((input_left, input_right), (output_left, output_right)) in
      input_channels.zip(output_channels)
    {
      let time_warp_output = self.time_warp.process(
        (input_left.get(), input_right.get()),
        &mut self.params,
        &mut self.notes.get_notes(),
      );
      output_left.set(time_warp_output.0);
      output_right.set(time_warp_output.1);
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
