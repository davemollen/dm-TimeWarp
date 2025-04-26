mod editor;
mod file_loader;
mod time_warp_parameters;
use file_loader::{BackgroundTask, FileLoader};
use nih_plug::prelude::*;
use std::sync::Arc;
use time_warp::{Notes, Params as ProcessParams, TimeMode, TimeWarp, WavFileData};
use time_warp_parameters::{TimeMode as ParamTimeMode, TimeWarpParameters};

struct DmTimeWarp {
  params: Arc<TimeWarpParameters>,
  time_warp: TimeWarp,
  process_params: ProcessParams,
  notes: Notes,
  file_loader: FileLoader,
}

impl Default for DmTimeWarp {
  fn default() -> Self {
    let sample_rate = 44100_f32;
    let params = Arc::new(TimeWarpParameters::default());
    let file_loader = FileLoader::new(sample_rate, params.file_path.clone());

    Self {
      params: params.clone(),
      time_warp: TimeWarp::new(sample_rate),
      process_params: ProcessParams::new(sample_rate),
      notes: Notes::new(),
      file_loader,
    }
  }
}

impl DmTimeWarp {
  pub fn set_param_values(&mut self, buffer_size: usize) {
    self.process_params.set(
      self.params.scan.value(),
      self.params.spray.value(),
      self.params.size.value(),
      self.params.speed.value(),
      self.params.density.value(),
      self.params.stretch.value(),
      self.params.record.value(),
      self.params.play.value(),
      match self.params.time_mode.value() {
        ParamTimeMode::Delay => TimeMode::Delay,
        ParamTimeMode::Looper => TimeMode::Looper,
      },
      self.params.time.value(),
      self.params.time_multiply.value(),
      self.params.highpass.value(),
      self.params.lowpass.value(),
      self.params.feedback.value(),
      self.params.recycle.value(),
      self.params.dry.value(),
      self.params.wet.value(),
      self.params.midi_enabled.value(),
      self.params.attack.value(),
      self.params.decay.value(),
      self.params.sustain.value(),
      self.params.release.value(),
      self.params.clear.value(),
      self.time_warp.get_delay_line(),
      buffer_size,
    );
    if self.process_params.should_clear_buffer() {
      *self.params.file_path.lock().unwrap() = "".to_string();
    }
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
}

impl Plugin for DmTimeWarp {
  const NAME: &'static str = "dm-TimeWarp";
  const VENDOR: &'static str = "DM";
  const URL: &'static str = "https://github.com/davemollen/dm-TimeWarp";
  const EMAIL: &'static str = "davemollen@gmail.com";
  const VERSION: &'static str = env!("CARGO_PKG_VERSION");

  const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
    main_input_channels: NonZeroU32::new(2),
    main_output_channels: NonZeroU32::new(2),
    ..AudioIOLayout::const_default()
  }];
  const MIDI_INPUT: MidiConfig = MidiConfig::Basic;
  const SAMPLE_ACCURATE_AUTOMATION: bool = true;

  type BackgroundTask = BackgroundTask;
  type SysExMessage = ();

  fn params(&self) -> Arc<dyn Params> {
    self.params.clone()
  }

  fn editor(&mut self, async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
    editor::create(
      self.params.clone(),
      self.params.editor_state.clone(),
      async_executor,
    )
  }

  fn initialize(
    &mut self,
    _audio_io_layout: &AudioIOLayout,
    buffer_config: &BufferConfig,
    context: &mut impl InitContext<Self>,
  ) -> bool {
    self.time_warp = TimeWarp::new(buffer_config.sample_rate);
    self.process_params = ProcessParams::new(buffer_config.sample_rate);
    self.file_loader.set_sample_rate(buffer_config.sample_rate);
    context.execute(BackgroundTask::LoadFile(
      self.params.file_path.lock().unwrap().clone(),
      false,
    ));

    true
  }

  fn task_executor(&mut self) -> TaskExecutor<Self> {
    let file_loader = self.file_loader.clone();

    Box::new(move |task| {
      file_loader.handle_task(task);
    })
  }

  fn process(
    &mut self,
    buffer: &mut Buffer,
    _aux: &mut AuxiliaryBuffers,
    context: &mut impl ProcessContext<Self>,
  ) -> ProcessStatus {
    self.set_param_values(buffer.samples());
    self.process_midi_events(context);

    if let Some(WavFileData { samples, duration }) = self.file_loader.try_receive_data() {
      self.time_warp.get_delay_line().set_values(&samples);
      self.process_params.set_file_duration(duration);
      self.process_params.reset_playback = true;
    }

    buffer.iter_samples().for_each(|mut channel_samples| {
      let channel_iterator = &mut channel_samples.iter_mut();
      let left_channel = channel_iterator.next().unwrap();
      let right_channel = channel_iterator.next().unwrap();

      (*left_channel, *right_channel) = self.time_warp.process(
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

impl ClapPlugin for DmTimeWarp {
  const CLAP_ID: &'static str = "dm-TimeWarp";
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

impl Vst3Plugin for DmTimeWarp {
  const VST3_CLASS_ID: [u8; 16] = *b"dm-TimeWarp.....";
  const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
    Vst3SubCategory::Fx,
    Vst3SubCategory::Delay,
    Vst3SubCategory::PitchShift,
  ];
}

nih_export_clap!(DmTimeWarp);
nih_export_vst3!(DmTimeWarp);
