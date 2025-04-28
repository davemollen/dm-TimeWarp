mod smooth;
mod stopwatch;
pub use smooth::Smoother;
use {
  crate::shared::float_ext::FloatExt,
  crate::stereo_delay_line::StereoDelayLine,
  smooth::{LinearSmooth, LogarithmicSmooth},
  stopwatch::Stopwatch,
};

#[derive(PartialEq)]
pub enum TimeMode {
  Delay,
  Looper,
}

pub struct Params {
  pub scan: f32,
  pub spray: f32,
  pub size: f32,
  pub speed: f32,
  pub density: f32,
  pub stretch: f32,
  pub recording_gain: LinearSmooth,
  pub playback_gain: LinearSmooth,
  pub time: LogarithmicSmooth,
  pub highpass: LinearSmooth,
  pub lowpass: LinearSmooth,
  pub feedback: LinearSmooth,
  pub recycle: LinearSmooth,
  pub dry: LinearSmooth,
  pub wet: LinearSmooth,
  pub midi_enabled: bool,
  is_initialized: bool,
  pub attack: LinearSmooth,
  pub decay: LinearSmooth,
  pub sustain: LinearSmooth,
  pub release: LinearSmooth,
  pub reset_playback: bool,
  prev_reset_playback: bool,
  file_duration: Option<f32>,
  loop_duration: Option<f32>,
  stopwatch: Stopwatch,
  prev_file_duration: Option<f32>,
  prev_play: bool,
  prev_clear: bool,
}

impl Params {
  pub fn new(sample_rate: f32) -> Self {
    Self {
      scan: 0.,
      spray: 0.,
      size: 0.,
      speed: 0.,
      density: 0.,
      stretch: 0.,
      recording_gain: LinearSmooth::new(sample_rate, 50.),
      playback_gain: LinearSmooth::new(sample_rate, 50.),
      time: LogarithmicSmooth::new(sample_rate, 0.3),
      highpass: LinearSmooth::new(sample_rate, 20.),
      lowpass: LinearSmooth::new(sample_rate, 20.),
      feedback: LinearSmooth::new(sample_rate, 20.),
      recycle: LinearSmooth::new(sample_rate, 20.),
      dry: LinearSmooth::new(sample_rate, 20.),
      wet: LinearSmooth::new(sample_rate, 20.),
      midi_enabled: false,
      is_initialized: false,
      attack: LinearSmooth::new(sample_rate, 20.),
      decay: LinearSmooth::new(sample_rate, 20.),
      sustain: LinearSmooth::new(sample_rate, 20.),
      release: LinearSmooth::new(sample_rate, 20.),
      reset_playback: false,
      prev_reset_playback: false,
      file_duration: None,
      loop_duration: None,
      stopwatch: Stopwatch::new(sample_rate),
      prev_file_duration: None,
      prev_play: true,
      prev_clear: false,
    }
  }

  pub fn set(
    &mut self,
    scan: f32,
    spray: f32,
    size: f32,
    speed: f32,
    density: f32,
    stretch: f32,
    record: bool,
    play: bool,
    time_mode: TimeMode,
    time: f32,
    time_multiply: f32,
    highpass: f32,
    lowpass: f32,
    feedback: f32,
    recycle: f32,
    dry: f32,
    wet: f32,
    midi_enabled: bool,
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    clear: bool,
    delay_line: &mut StereoDelayLine,
    buffer_size: usize,
  ) {
    if self.prev_reset_playback {
      self.reset_playback = false;
    }
    self.scan = scan;
    self.spray = spray;
    self.size = size.powf(0.333);
    self.speed = speed;
    self.density = density * density;
    self.stretch = stretch;
    self.midi_enabled = midi_enabled;

    let overridden_play = self.override_play(play, &time_mode);
    let recording_gain = if record { 1. } else { 0. };
    let playback_gain = if overridden_play { 1. } else { 0. };
    let sustain = sustain.dbtoa();
    let dry = dry.dbtoa();
    let wet = wet.dbtoa();

    if clear && !self.prev_clear {
      self.file_duration = None;
      self.stopwatch.reset();
      self.loop_duration = None;
      delay_line.reset();
      self.prev_clear = false;
    }

    if self.is_initialized {
      self.recording_gain.set_target(recording_gain);
      self.playback_gain.set_target(playback_gain);
      self.set_time(time_mode, record, play, time, time_multiply, buffer_size);
      self.highpass.set_target(highpass);
      self.lowpass.set_target(lowpass);
      self.feedback.set_target(feedback);
      self.recycle.set_target(recycle);
      self.dry.set_target(dry);
      self.wet.set_target(wet);
      self.attack.set_target(attack);
      self.decay.set_target(decay);
      self.sustain.set_target(sustain);
      self.release.set_target(release);
    } else {
      self.recording_gain.reset(recording_gain);
      self.playback_gain.reset(playback_gain);
      self.reset_time(time, time_multiply);
      self.highpass.reset(highpass);
      self.lowpass.reset(lowpass);
      self.feedback.reset(feedback);
      self.recycle.reset(recycle);
      self.dry.reset(dry);
      self.wet.reset(wet);
      self.attack.reset(attack);
      self.decay.reset(decay);
      self.sustain.reset(sustain);
      self.release.reset(release);
      self.is_initialized = true;
    }
    self.prev_play = play;
    self.prev_clear = clear;
    self.prev_file_duration = self.file_duration;
    self.prev_reset_playback = self.reset_playback;
  }

  pub fn set_file_duration(&mut self, file_duration: f32) {
    self.file_duration = Some(file_duration);
  }

  pub fn should_clear_buffer(&mut self) -> bool {
    self.prev_clear
  }

  fn override_play(&mut self, play: bool, time_mode: &TimeMode) -> bool {
    match (play, time_mode, self.loop_duration, self.file_duration) {
      (true, TimeMode::Looper, None, None) => false,
      (true, _, _, _) => {
        // reset playback to beginning if play was off previously
        self.reset_playback = !self.prev_play;
        true
      }
      (false, _, _, _) => false,
    }
  }

  fn set_time(
    &mut self,
    time_mode: TimeMode,
    record: bool,
    play: bool,
    time: f32,
    time_multiply: f32,
    buffer_size: usize,
  ) {
    match (
      self.file_duration,
      self.prev_file_duration,
      self.loop_duration,
      time_mode,
    ) {
      (Some(file_duration), Some(prev_file_duration), _, _) => {
        if file_duration == prev_file_duration {
          self.time.set_target(file_duration * time_multiply);
        } else {
          self.time.reset(file_duration * time_multiply);
        }
      }
      (Some(file_duration), None, _, _) => {
        self.time.reset(file_duration * time_multiply);
      }
      (None, _, None, TimeMode::Looper) => {
        // stop stopwatch if play changed from false to true
        let start = record && !(!self.prev_play && play);
        if let Some(loop_duration) = self.stopwatch.process(start, buffer_size) {
          self.time.reset(loop_duration * time_multiply);
          self.loop_duration = Some(loop_duration);
        }
      }
      (_, _, Some(loop_duration), TimeMode::Looper) => {
        self.time.set_target(loop_duration * time_multiply);
      }
      _ => {
        self.time.set_target(time);
      }
    }
  }

  fn reset_time(&mut self, time: f32, time_multiply: f32) {
    match (self.file_duration, self.loop_duration) {
      (Some(dur), _) => self.time.reset(dur * time_multiply),
      (None, Some(dur)) => self.time.reset(dur * time_multiply),
      _ => {
        self.time.reset(time);
      }
    }
  }
}
