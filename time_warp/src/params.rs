mod smooth;
mod stopwatch;
pub use smooth::Smoother;
use {
  crate::shared::float_ext::FloatExt,
  smooth::{LinearSmooth, LogarithmicSmooth},
  stopwatch::Stopwatch,
};

#[derive(PartialEq)]
pub enum RecordMode {
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
  pub filter_coefficients: ([f32; 3], [f32; 3]),
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
  prev_flush: bool,
  is_flushing: bool,
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
      filter_coefficients: ([0.; 3], [0.; 3]),
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
      prev_flush: false,
      is_flushing: false,
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
    record_mode: RecordMode,
    time: f32,
    length: f32,
    feedback: f32,
    recycle: f32,
    dry: f32,
    wet: f32,
    midi_enabled: bool,
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    flush: bool,
    buffer_size: usize,
  ) {
    if self.prev_reset_playback {
      self.reset_playback = false;
    }
    self.scan = scan;
    self.spray = spray;
    self.set_size(size);
    self.speed = speed;
    self.density = density;
    self.stretch = stretch;
    self.midi_enabled = midi_enabled;

    let overridden_play = self.override_play(play, &record_mode);
    let recording_gain = if record { 1. } else { 0. };
    let playback_gain = if overridden_play { 1. } else { 0. };
    let dry = dry.fast_dbtoa();
    let wet = wet.fast_dbtoa();

    if flush && !self.prev_flush {
      self.file_duration = None;
      self.stopwatch.reset();
      self.loop_duration = None;
      self.is_flushing = true;
    } else {
      self.is_flushing = false;
    }

    if self.is_initialized {
      self.recording_gain.set_target(recording_gain);
      self.playback_gain.set_target(playback_gain);
      self.set_time(record_mode, record, play, time, length, buffer_size);
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
      self.reset_time(time, length);
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
    self.prev_flush = flush;
    self.prev_file_duration = self.file_duration;
    self.prev_reset_playback = self.reset_playback;
  }

  pub fn set_file_duration(&mut self, file_duration: f32) {
    self.file_duration = Some(file_duration);
  }

  pub fn set_reset_playback(&mut self, reset_playback: bool) {
    self.reset_playback = reset_playback;
    self.recording_gain.reset(0.);
  }

  pub fn should_clear_buffer(&mut self) -> bool {
    self.is_flushing
  }

  fn override_play(&mut self, play: bool, record_mode: &RecordMode) -> bool {
    match (play, record_mode, self.loop_duration, self.file_duration) {
      (true, RecordMode::Looper, None, None) => false,
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
    record_mode: RecordMode,
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
      record_mode,
    ) {
      (Some(file_duration), Some(prev_file_duration), _, _) => {
        if file_duration == prev_file_duration {
          self.time.set_target(file_duration * length);
        } else {
          self.time.reset(file_duration * length);
        }
      }
      (Some(file_duration), None, _, _) => {
        self.time.reset(file_duration * length);
      }
      (None, _, None, RecordMode::Looper) => {
        // stop stopwatch if play changed from false to true
        let start = record && !(!self.prev_play && play);
        if let Some(loop_duration) = self.stopwatch.process(start, buffer_size) {
          self.time.reset(loop_duration * length);
          self.loop_duration = Some(loop_duration);
          self.reset_playback = true;
        }
      }
      (_, _, Some(loop_duration), RecordMode::Looper) => {
        self.time.set_target(loop_duration * length);
      }
      _ => {
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

  fn set_size(&mut self, size: f32) {
    // same as size.powf(0.333) and size.cbrt()
    self.size = if size == 0. {
      0.
    } else if size == 1. {
      1.
    } else {
      size.fast_cbrt()
    };
  }
}
