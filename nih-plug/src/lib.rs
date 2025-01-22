use grain_stretch::{GrainStretch, Params as ProcessParams};
mod grain_stretch_parameters;
use grain_stretch_parameters::GrainStretchParameters;
use nih_plug::prelude::*;
use std::sync::Arc;
mod editor;

struct DmGrainStretch {
  params: Arc<GrainStretchParameters>,
  grain_stretch: GrainStretch,
  process_params: ProcessParams,
}

impl Default for DmGrainStretch {
  fn default() -> Self {
    let params = Arc::new(GrainStretchParameters::default());
    Self {
      params: params.clone(),
      grain_stretch: GrainStretch::new(44100.),
      process_params: ProcessParams::new(44100.),
    }
  }
}

impl Plugin for DmGrainStretch {
  const NAME: &'static str = "dm-GrainStretch";
  const VENDOR: &'static str = "DM";
  const URL: &'static str = "https://github.com/davemollen/dm-GrainStretch";
  const EMAIL: &'static str = "davemollen@gmail.com";
  const VERSION: &'static str = env!("CARGO_PKG_VERSION");

  const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
    main_input_channels: NonZeroU32::new(2),
    main_output_channels: NonZeroU32::new(2),
    ..AudioIOLayout::const_default()
  }];
  const MIDI_INPUT: MidiConfig = MidiConfig::None;
  const SAMPLE_ACCURATE_AUTOMATION: bool = true;

  // More advanced plugins can use this to run expensive background tasks. See the field's
  // documentation for more information. `()` means that the plugin does not have any background
  // tasks.
  type BackgroundTask = ();
  type SysExMessage = ();

  fn params(&self) -> Arc<dyn Params> {
    self.params.clone()
  }

  fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
    editor::create(self.params.clone(), self.params.editor_state.clone())
  }

  fn initialize(
    &mut self,
    _audio_io_layout: &AudioIOLayout,
    buffer_config: &BufferConfig,
    _context: &mut impl InitContext<Self>,
  ) -> bool {
    self.grain_stretch = GrainStretch::new(buffer_config.sample_rate);
    self.process_params = ProcessParams::new(buffer_config.sample_rate);
    true
  }

  fn process(
    &mut self,
    buffer: &mut Buffer,
    _aux: &mut AuxiliaryBuffers,
    _context: &mut impl ProcessContext<Self>,
  ) -> ProcessStatus {
    self.process_params.set(
      self.params.scan.value(),
      self.params.spray.value(),
      self.params.size.value(),
      self.params.speed.value(),
      self.params.density.value(),
      self.params.stretch.value(),
      if self.params.record.value() { 1. } else { 0. },
      self.params.time.value(),
      self.params.highpass.value(),
      self.params.lowpass.value(),
      self.params.overdub.value(),
      self.params.recycle.value(),
      self.params.dry.value(),
      self.params.wet.value(),
    );

    buffer.iter_samples().for_each(|mut channel_samples| {
      let channel_iterator = &mut channel_samples.iter_mut();
      let left_channel = channel_iterator.next().unwrap();
      let right_channel = channel_iterator.next().unwrap();

      (*left_channel, *right_channel) = self
        .grain_stretch
        .process((*left_channel, *right_channel), &mut self.process_params);
    });
    ProcessStatus::Normal
  }

  // This can be used for cleaning up special resources like socket connections whenever the
  // plugin is deactivated. Most plugins won't need to do anything here.
  fn deactivate(&mut self) {}
}

impl ClapPlugin for DmGrainStretch {
  const CLAP_ID: &'static str = "dm-GrainStretch";
  const CLAP_DESCRIPTION: Option<&'static str> = Some("A granular plugin");
  const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
  const CLAP_SUPPORT_URL: Option<&'static str> = None;
  const CLAP_FEATURES: &'static [ClapFeature] = &[
    ClapFeature::AudioEffect,
    ClapFeature::Stereo,
    ClapFeature::Mono,
    ClapFeature::Granular,
    ClapFeature::PitchShifter,
    ClapFeature::Delay,
  ];
}

impl Vst3Plugin for DmGrainStretch {
  const VST3_CLASS_ID: [u8; 16] = *b"dm-GrainStretch.";
  const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
    Vst3SubCategory::Fx,
    Vst3SubCategory::Delay,
    Vst3SubCategory::PitchShift,
  ];
}

nih_export_clap!(DmGrainStretch);
nih_export_vst3!(DmGrainStretch);
