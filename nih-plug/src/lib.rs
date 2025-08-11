mod editor;
mod time_warp_parameters;
mod worker;
use nih_plug::prelude::*;
use std::sync::Arc;
use time_warp::{AudioFileData, Notes, Params as ProcessParams, RecordMode, TimeWarp};
use time_warp_parameters::{RecordMode as ParamRecordMode, TimeWarpParameters};
use worker::{Worker, WorkerRequest, WorkerResponseData};

pub struct DmTimeWarp {
  params: Arc<TimeWarpParameters>,
  time_warp: TimeWarp,
  process_params: ProcessParams,
  notes: Notes,
  worker: Worker,
}

impl Default for DmTimeWarp {
  fn default() -> Self {
    let sample_rate = 44100_f32;
    let params = Arc::new(TimeWarpParameters::default());
    let time_warp = TimeWarp::new(sample_rate);

    Self {
      params: params.clone(),
      time_warp: TimeWarp::new(sample_rate),
      process_params: ProcessParams::new(sample_rate),
      notes: Notes::new(),
      worker: Worker::new(
        sample_rate,
        params.file_path.clone(),
        time_warp.get_delay_line_size(),
      ),
    }
  }
}

impl DmTimeWarp {
  pub fn set_param_values(&mut self, buffer_size: usize, context: &mut impl ProcessContext<Self>) {
    self.process_params.set(
      self.params.scan.value(),
      self.params.spray.value(),
      self.params.size.value(),
      self.params.speed.value(),
      self.params.density.value(),
      self.params.stretch.value(),
      self.params.stereo.value(),
      self.params.record.value(),
      self.params.play.value(),
      match self.params.record_mode.value() {
        ParamRecordMode::Delay => RecordMode::Delay,
        ParamRecordMode::Looper => RecordMode::Looper,
      },
      self.params.time.value(),
      self.params.length.value(),
      self.params.recycle.value(),
      self.params.feedback.value(),
      self.params.dry.value(),
      self.params.wet.value(),
      self.params.midi_enabled.value(),
      self.params.attack.value(),
      self.params.decay.value(),
      self.params.sustain.value(),
      self.params.release.value(),
      self.params.erase.value(),
      buffer_size,
    );

    self
      .time_warp
      .get_filter()
      .set_coefficients(self.params.highpass.value(), self.params.lowpass.value());

    if self.process_params.should_erase_buffer() {
      *self.params.file_path.lock().unwrap() = "".to_string();
      context.execute_background(WorkerRequest::FlushBuffer);
    }

    self
      .notes
      .set_voice_count(self.params.voices.value() as usize);
  }

  pub fn process_midi_events(&mut self, context: &mut impl ProcessContext<Self>) {
    if self.process_params.midi_enabled {
      while let Some(event) = context.next_event() {
        match event {
          NoteEvent::NoteOn { note, velocity, .. } => {
            self.notes.note_on(note, velocity);
          }
          NoteEvent::NoteOff { note, .. } => {
            self.notes.note_off(note);
          }
          NoteEvent::MidiCC { cc, value, .. } => match cc {
            64 => self.notes.sustain(value > 0.),
            120 => self.notes.remove_notes(),
            123 => self.notes.release_notes(),
            _ => (),
          },
          NoteEvent::MidiPitchBend { value, .. } => {
            let pitchbend_factor = 2f32.powf(value * 2. - 1.);
            self.process_params.set_pitch_bend_factor(pitchbend_factor);
          }
          _ => (),
        }
      }
    } else {
      self.notes.remove_notes();
    }
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
  const MIDI_INPUT: MidiConfig = MidiConfig::MidiCCs;
  const SAMPLE_ACCURATE_AUTOMATION: bool = true;

  type BackgroundTask = WorkerRequest;
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
    self.worker.initialize(
      buffer_config.sample_rate,
      self.time_warp.get_delay_line_size(),
    );
    context.execute(WorkerRequest::LoadFile(
      self.params.file_path.lock().unwrap().clone(),
      false,
    ));

    true
  }

  fn task_executor(&mut self) -> TaskExecutor<Self> {
    let worker = self.worker.clone();

    Box::new(move |task| {
      worker.handle_task(task);
    })
  }

  fn process(
    &mut self,
    buffer: &mut Buffer,
    _aux: &mut AuxiliaryBuffers,
    context: &mut impl ProcessContext<Self>,
  ) -> ProcessStatus {
    self.set_param_values(buffer.samples(), context);
    self.process_midi_events(context);

    if let Some(worker_response_data) = self.worker.try_receive_data() {
      match worker_response_data {
        WorkerResponseData::LoadFile(AudioFileData {
          samples,
          duration_in_samples,
          duration_in_ms,
        }) => {
          self
            .time_warp
            .set_delay_line_values(samples, duration_in_samples);
          self.process_params.set_file_duration(duration_in_ms);
          self.process_params.set_reset_playback(true);
        }
        WorkerResponseData::FlushBuffer(samples) => {
          self.time_warp.set_delay_line_values(samples, 0);
        }
      }
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
