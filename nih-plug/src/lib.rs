use grain_stretch::{GrainStretch, Notes, Params as ProcessParams, WavProcessor};
mod grain_stretch_parameters;
use grain_stretch_parameters::GrainStretchParameters;
use nih_plug::prelude::*;
use std::sync::Arc;
mod editor;

struct DmGrainStretch {
  params: Arc<GrainStretchParameters>,
  grain_stretch: GrainStretch,
  process_params: ProcessParams,
  notes: Notes,
  wav_processor: WavProcessor,
  loaded_file_path: String,
}

impl Default for DmGrainStretch {
  fn default() -> Self {
    let sample_rate = 44100_f32;
    let params = Arc::new(GrainStretchParameters::default());
    Self {
      params: params.clone(),
      grain_stretch: GrainStretch::new(sample_rate),
      process_params: ProcessParams::new(sample_rate),
      notes: Notes::new(),
      wav_processor: WavProcessor::new(sample_rate),
      loaded_file_path: String::new(),
    }
  }
}

impl DmGrainStretch {
  pub fn set_param_values(&mut self) {
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
      self.params.midi_enabled.value(),
      self.params.attack.value(),
      self.params.decay.value(),
      self.params.sustain.value(),
      self.params.release.value(),
    );
  }

  pub fn process_midi_events(&mut self, context: &mut impl ProcessContext<Self>) {
    while let Some(event) = context.next_event() {
      match event {
        NoteEvent::NoteOn { note, velocity, .. } => {
          self.notes.note_on(note, velocity);
        }
        NoteEvent::NoteOff { note, .. } => {
          self.notes.note_off(note);
        }
        _ => (),
      }
    }

    self
      .notes
      .set_voice_count(self.params.voices.value() as usize);
  }

  pub fn load_wav_file(&mut self, is_initializing: bool) {
    let path = self.params.file_path.lock().unwrap().clone();
    if path.is_empty()
      || if is_initializing {
        false
      } else {
        self.loaded_file_path == path
      }
    {
      return;
    }
    match self.wav_processor.read_wav(&path) {
      Ok(samples) => {
        self.grain_stretch.load_wav_file(samples);
      }
      Err(err) => nih_log!("Failed to load WAV file: {:?}", err),
    };
    self.loaded_file_path = path;
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
  const MIDI_INPUT: MidiConfig = MidiConfig::Basic;
  const SAMPLE_ACCURATE_AUTOMATION: bool = true;

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
    self.wav_processor = WavProcessor::new(buffer_config.sample_rate);
    self.load_wav_file(true);

    true
  }

  fn process(
    &mut self,
    buffer: &mut Buffer,
    _aux: &mut AuxiliaryBuffers,
    context: &mut impl ProcessContext<Self>,
  ) -> ProcessStatus {
    self.set_param_values();
    self.process_midi_events(context);
    self.load_wav_file(false);

    buffer.iter_samples().for_each(|mut channel_samples| {
      let channel_iterator = &mut channel_samples.iter_mut();
      let left_channel = channel_iterator.next().unwrap();
      let right_channel = channel_iterator.next().unwrap();

      (*left_channel, *right_channel) = self.grain_stretch.process(
        (*left_channel, *right_channel),
        &mut self.process_params,
        &mut self.notes.get_notes(),
      );
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
