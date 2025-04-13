mod smooth;
mod stopwatch;
pub use smooth::Smoother;
use {
  crate::shared::float_ext::FloatExt,
  smooth::{ExponentialSmooth, LinearSmooth, LogarithmicSmooth},
  stopwatch::Stopwatch,
};

#[derive(PartialEq)]
pub enum TimeMode {
  Delay,
  Looper,
}

pub struct Params {
  pub recording_gain: ExponentialSmooth,
  pub scan: f32,
  pub spray: f32,
  pub size: f32,
  pub speed: f32,
  pub density: f32,
  pub stretch: f32,
  pub time: LogarithmicSmooth,
  pub highpass: LinearSmooth,
  pub lowpass: LinearSmooth,
  pub overdub: LinearSmooth,
  pub recycle: LinearSmooth,
  pub dry: ExponentialSmooth,
  pub wet: ExponentialSmooth,
  pub midi_enabled: bool,
  is_initialized: bool,
  pub attack: LinearSmooth,
  pub decay: LinearSmooth,
  pub sustain: LinearSmooth,
  pub release: LinearSmooth,
  file_duration: Option<f32>,
  loop_duration: Option<f32>,
  stopwatch: Stopwatch,
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
      recording_gain: ExponentialSmooth::new(sample_rate, 20.),
      time: LogarithmicSmooth::new(sample_rate, 0.3),
      highpass: LinearSmooth::new(sample_rate, 20.),
      lowpass: LinearSmooth::new(sample_rate, 20.),
      overdub: LinearSmooth::new(sample_rate, 20.),
      recycle: LinearSmooth::new(sample_rate, 20.),
      dry: ExponentialSmooth::new(sample_rate, 20.),
      wet: ExponentialSmooth::new(sample_rate, 20.),
      midi_enabled: false,
      is_initialized: false,
      attack: LinearSmooth::new(sample_rate, 20.),
      decay: LinearSmooth::new(sample_rate, 20.),
      sustain: LinearSmooth::new(sample_rate, 20.),
      release: LinearSmooth::new(sample_rate, 20.),
      file_duration: None,
      loop_duration: None,
      stopwatch: Stopwatch::new(sample_rate),
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
    time_mode: TimeMode,
    time: f32,
    time_multiply: f32,
    highpass: f32,
    lowpass: f32,
    overdub: f32,
    recycle: f32,
    dry: f32,
    wet: f32,
    midi_enabled: bool,
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    file_duration: Option<f32>,
    clear: bool,
  ) {
    self.scan = scan;
    self.spray = spray;
    self.size = size.powf(0.333);
    self.speed = speed;
    self.density = density * density;
    self.stretch = stretch;
    self.midi_enabled = midi_enabled;

    let recording_gain = if record { 1. } else { 0. };
    let sustain = sustain.dbtoa();
    let dry = dry.dbtoa();
    let wet = wet.dbtoa();

    if clear {
      self.stopwatch.reset();
      self.loop_duration = None;
    }

    if self.is_initialized {
      self.recording_gain.set_target(recording_gain);
      self.set_time(file_duration, time_mode, record, time, time_multiply);
      self.highpass.set_target(highpass);
      self.lowpass.set_target(lowpass);
      self.overdub.set_target(overdub);
      self.recycle.set_target(recycle);
      self.dry.set_target(dry);
      self.wet.set_target(wet);
      self.attack.set_target(attack);
      self.decay.set_target(decay);
      self.sustain.set_target(sustain);
      self.release.set_target(release);
    } else {
      self.recording_gain.reset(recording_gain);
      self.reset_time(file_duration, time, time_multiply);
      self.highpass.reset(highpass);
      self.lowpass.reset(lowpass);
      self.overdub.reset(overdub);
      self.recycle.reset(recycle);
      self.dry.reset(dry);
      self.wet.reset(wet);
      self.attack.reset(attack);
      self.decay.reset(decay);
      self.sustain.reset(sustain);
      self.release.reset(release);
      self.is_initialized = true;
    }

    self.file_duration = file_duration;
  }

  fn set_time(
    &mut self,
    file_duration: Option<f32>,
    time_mode: TimeMode,
    record: bool,
    time: f32,
    time_multiply: f32,
  ) {
    match (
      file_duration,
      self.file_duration,
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
      (_, _, None, TimeMode::Looper) => {
        if let Some(loop_duration) = self.stopwatch.process(record) {
          self.time.reset(loop_duration * time_multiply);
          self.loop_duration = Some(loop_duration);
        }
      }
      (_, _, Some(loop_duration), TimeMode::Looper) => {
        self.time.set_target(loop_duration * time_multiply);
      }
      (_, _, _, _) => {
        self.time.set_target(time);
      }
    }
  }

  fn reset_time(&mut self, file_duration: Option<f32>, time: f32, time_multiply: f32) {
    match (file_duration, self.loop_duration) {
      (Some(dur), _) => self.time.reset(dur * time_multiply),
      (None, Some(dur)) => self.time.reset(dur * time_multiply),
      (None, None) => {
        self.time.reset(time);
      }
    }
  }
}
