mod smooth;
pub use smooth::Smoother;
use smooth::{ExponentialSmooth, LinearSmooth, LogarithmicSmooth};

use crate::shared::float_ext::FloatExt;

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
  is_initialized: bool,
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
      is_initialized: false,
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
    recording_gain: f32,
    time: f32,
    highpass: f32,
    lowpass: f32,
    overdub: f32,
    recycle: f32,
    dry: f32,
    wet: f32,
  ) {
    self.scan = scan;
    self.spray = spray;
    self.size = size.powf(0.25);
    self.speed = speed;
    self.density = density * density;
    self.stretch = stretch;

    let dry = dry.dbtoa();
    let wet = wet.dbtoa();

    if self.is_initialized {
      self.recording_gain.set_target(recording_gain);
      self.time.set_target(time);
      self.highpass.set_target(highpass);
      self.lowpass.set_target(lowpass);
      self.overdub.set_target(overdub);
      self.recycle.set_target(recycle);
      self.dry.set_target(dry);
      self.wet.set_target(wet);
    } else {
      self.recording_gain.reset(recording_gain);
      self.time.reset(time);
      self.highpass.reset(highpass);
      self.lowpass.reset(lowpass);
      self.overdub.reset(overdub);
      self.recycle.reset(recycle);
      self.dry.reset(dry);
      self.wet.reset(wet);
      self.is_initialized = true;
    }
  }
}
