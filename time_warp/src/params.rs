mod phasor;
mod smooth;
mod stopwatch;
pub use smooth::Smoother;
use {
  crate::shared::float_ext::FloatExt,
  phasor::Phasor,
  smooth::{ExponentialSmooth, LinearSmooth},
  stopwatch::Stopwatch,
};

#[derive(Clone, Copy, PartialEq)]
pub enum SampleMode {
  Delay,
  Looper,
  Sampler,
}

pub struct Params {
  pub scan: f32,
  pub spray: f32,
  pub size: f32,
  pub density: f32,
  pub stereo: f32,
  pub speed: f32,
  pub stretch: f32,
  pub should_reset_start_offset: bool,
  pub recording_gain: LinearSmooth,
  pub playback_gain: LinearSmooth,
  pub time: ExponentialSmooth,
  pub filter_coefficients: ([f32; 3], [f32; 3]),
  pub recycle: LinearSmooth,
  pub feedback: LinearSmooth,
  pub dry: LinearSmooth,
  pub wet: LinearSmooth,
  pub midi_enabled: bool,
  is_initialized: bool,
  pub attack: LinearSmooth,
  pub decay: LinearSmooth,
  pub sustain: LinearSmooth,
  pub release: LinearSmooth,
  pub reset_playback: bool,
  pub start_offset_phase: f32,
  file_duration: Option<f32>,
  loop_duration: Option<f32>,
  has_delay_mode_recording: bool,
  stopwatch: Stopwatch,
  prev_file_duration: Option<f32>,
  prev_play: bool,
  prev_erase: bool,
  is_erasing_buffer: bool,
  prev_sample_mode: Option<SampleMode>,
  pitch_bend_factor: f32,
  start_offset_phasor: Phasor,
}

impl Params {
  pub fn new(sample_rate: f32) -> Self {
    Self {
      scan: 0.,
      spray: 0.,
      size: 1.,
      density: 0.,
      stereo: 1.,
      speed: 1.,
      stretch: 0.,
      should_reset_start_offset: false,
      recording_gain: LinearSmooth::new(sample_rate, 55.),
      playback_gain: LinearSmooth::new(sample_rate, 55.),
      time: ExponentialSmooth::new(sample_rate, 2.109),
      filter_coefficients: ([0.; 3], [0.; 3]),
      recycle: LinearSmooth::new(sample_rate, 20.),
      feedback: LinearSmooth::new(sample_rate, 20.),
      dry: LinearSmooth::new(sample_rate, 20.),
      wet: LinearSmooth::new(sample_rate, 20.),
      midi_enabled: false,
      is_initialized: false,
      attack: LinearSmooth::new(sample_rate, 20.),
      decay: LinearSmooth::new(sample_rate, 20.),
      sustain: LinearSmooth::new(sample_rate, 20.),
      release: LinearSmooth::new(sample_rate, 20.),
      reset_playback: false,
      start_offset_phase: 0.,
      file_duration: None,
      loop_duration: None,
      has_delay_mode_recording: false,
      stopwatch: Stopwatch::new(sample_rate),
      prev_file_duration: None,
      prev_play: true,
      prev_erase: false,
      is_erasing_buffer: false,
      prev_sample_mode: None,
      pitch_bend_factor: 1.,
      start_offset_phasor: Phasor::new(sample_rate),
    }
  }

  pub fn set(
    &mut self,
    scan: f32,
    spray: f32,
    size: f32,
    density: f32,
    stereo: f32,
    pitch: f32,
    stretch: f32,
    record: bool,
    play: bool,
    sample_mode: SampleMode,
    time: f32,
    length: f32,
    recycle: f32,
    feedback: f32,
    dry: f32,
    wet: f32,
    midi_enabled: bool,
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    erase: bool,
    buffer_size: usize,
  ) {
    self.scan = scan;
    self.spray = spray;
    self.size = size;
    self.density = density;
    self.stereo = stereo;
    self.speed = 2_f32.powf(pitch / 12.)
      * if midi_enabled {
        self.pitch_bend_factor
      } else {
        1.
      };
    self.stretch = stretch;
    self.midi_enabled = midi_enabled;

    let sample_mode_has_changed = self
      .prev_sample_mode
      .map_or(false, |prev_sample_mode| sample_mode != prev_sample_mode);
    let erase_has_changed = erase && !self.prev_erase;
    self.is_erasing_buffer = sample_mode_has_changed || erase_has_changed;

    if self.is_erasing_buffer {
      self.has_delay_mode_recording = false;
      self.should_reset_start_offset = true;
      self.reset_playback = true;
      self.file_duration = None;
      self.stopwatch.reset();
      self.loop_duration = None;
    }

    if sample_mode_has_changed && sample_mode == SampleMode::Looper {
      self.playback_gain.reset(0.);
    }

    let overridden_play = self.override_play(play, &sample_mode);
    let recording_gain = if record { 1. } else { 0. };
    let playback_gain = if overridden_play { 1. } else { 0. };
    let dry = dry.fast_dbtoa();
    let wet = wet.fast_dbtoa();

    if self.is_initialized {
      self.recording_gain.set_target(recording_gain);
      self.playback_gain.set_target(playback_gain);
      self.set_time(sample_mode, record, play, time, length, buffer_size);
      self.recycle.set_target(recycle);
      self.feedback.set_target(feedback);
      self.dry.set_target(dry);
      self.wet.set_target(wet);
      self.attack.set_target(attack);
      self.decay.set_target(decay);
      self.sustain.set_target(sustain);
      self.release.set_target(release);
    } else {
      self.recording_gain.reset(recording_gain);
      self.playback_gain.reset(playback_gain);
      self.reset_time(time, length);
      self.recycle.reset(recycle);
      self.feedback.reset(feedback);
      self.dry.reset(dry);
      self.wet.reset(wet);
      self.attack.reset(attack);
      self.decay.reset(decay);
      self.sustain.reset(sustain);
      self.release.reset(release);
      self.is_initialized = true;
    }
    if sample_mode == SampleMode::Looper && self.loop_duration.is_none() {
      self.feedback.reset(0.);
    } else if self.prev_sample_mode == Some(SampleMode::Looper) {
      self.feedback.reset(feedback);
    }

    self.set_start_offset_phase(buffer_size);

    self.prev_play = play;
    self.prev_erase = erase;
    self.prev_file_duration = self.file_duration;
    self.prev_sample_mode = Some(sample_mode);
  }

  pub fn settle(&mut self) {
    if self.reset_playback {
      self.reset_playback = false;
    }

    if self.should_reset_start_offset {
      self.should_reset_start_offset = false;
    }
  }

  pub fn set_file_duration(&mut self, file_duration: f32) {
    self.file_duration = Some(file_duration);
  }

  pub fn set_reset_playback(&mut self, reset_playback: bool) {
    self.reset_playback = reset_playback;
    self.recording_gain.reset(0.);
  }

  pub fn should_erase_buffer(&mut self) -> bool {
    self.is_erasing_buffer
  }

  pub fn set_pitch_bend_factor(&mut self, pitch_bend_factor: f32) {
    self.pitch_bend_factor = pitch_bend_factor;
  }

  pub fn get_target_time(&self) -> f32 {
    self.time.get_target()
  }

  fn override_play(&mut self, play: bool, sample_mode: &SampleMode) -> bool {
    match (play, sample_mode, self.loop_duration, self.file_duration) {
      (true, SampleMode::Looper, None, None) => false,
      (true, _, _, _) => {
        // reset playback to beginning if play was off previously and reset playback isn't activated already
        if !self.reset_playback {
          self.reset_playback = !self.prev_play;
        }
        true
      }
      (false, _, _, _) => false,
    }
  }

  fn set_time(
    &mut self,
    sample_mode: SampleMode,
    record: bool,
    play: bool,
    time: f32,
    length: f32,
    buffer_size: usize,
  ) {
    match (
      self.file_duration,
      self.prev_file_duration,
      self.loop_duration,
      sample_mode,
    ) {
      (Some(file_duration), Some(prev_file_duration), _, _) => {
        if file_duration == prev_file_duration {
          self.time.set_target(file_duration * length);
        } else {
          self.time.reset(file_duration * length);
          self.should_reset_start_offset = true;
        }
      }
      (Some(file_duration), None, _, _) => {
        self.time.reset(file_duration * length);
        self.should_reset_start_offset = true;
      }
      (None, _, None, SampleMode::Looper) => {
        // stop stopwatch if play changed from false to true
        let start = record && !(!self.prev_play && play);
        if let Some(loop_duration) = self.stopwatch.process(start, buffer_size) {
          self.time.reset(loop_duration * length);
          self.loop_duration = Some(loop_duration);
          self.reset_playback = true;
          self.should_reset_start_offset = true;
        }
      }
      (_, _, Some(loop_duration), SampleMode::Looper) => {
        self.time.set_target(loop_duration * length);
      }
      _ => {
        if record && !self.has_delay_mode_recording {
          self.has_delay_mode_recording = true;
          self.should_reset_start_offset = true;
        }
        self.time.set_target(time);
      }
    }
  }

  fn reset_time(&mut self, time: f32, length: f32) {
    match (self.file_duration, self.loop_duration) {
      (Some(dur), _) => self.time.reset(dur * length),
      (None, Some(dur)) => self.time.reset(dur * length),
      _ => {
        self.time.reset(time);
      }
    }
  }

  fn set_start_offset_phase(&mut self, buffer_size: usize) {
    if self.should_reset_start_offset {
      self.start_offset_phasor.reset();
    }
    let time = self.time.get_target();
    self.start_offset_phase = 1.
      - self
        .start_offset_phasor
        .process(if time == 0. { 0. } else { 1000. / time }, buffer_size);
  }
}
