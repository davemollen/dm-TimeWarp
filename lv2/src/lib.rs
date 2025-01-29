extern crate grain_stretch;
extern crate lv2;
use grain_stretch::{GrainStretch, Params};
use lv2::prelude::*;

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
}

#[uri("https://github.com/davemollen/dm-GrainStretch")]
struct DmGrainStretch {
  grain_stretch: GrainStretch,
  params: Params,
}

impl Plugin for DmGrainStretch {
  // Tell the framework which ports this plugin has.
  type Ports = Ports;

  // We don't need any special host features; We can leave them out.
  type InitFeatures = ();
  type AudioFeatures = ();

  // Create a new instance of the plugin; Trivial in this case.
  fn new(plugin_info: &PluginInfo, _features: &mut ()) -> Option<Self> {
    let sample_rate = plugin_info.sample_rate() as f32;

    Some(Self {
      grain_stretch: GrainStretch::new(sample_rate),
      params: Params::new(sample_rate),
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
        .process((*input_left, *input_right), &mut self.params);
    }
  }
}

// Generate the plugin descriptor function which exports the plugin to the outside world.
lv2_descriptors!(DmGrainStretch);
