mod smooth;
pub use smooth::Smoother;
use smooth::{ExponentialSmooth, LinearSmooth};

pub struct Params {
  pub pitch: f32,
  pub size: f32,
  pub scan: f32,
  pub density: f32,
  pub stretch: f32,
  pub time: LinearSmooth,
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
      pitch: 0.,
      size: 0.,
      scan: 0.,
      density: 0.,
      stretch: 0.,
      time: LinearSmooth::new(sample_rate, 12.),
      highpass: LinearSmooth::new(sample_rate, 12.),
      lowpass: LinearSmooth::new(sample_rate, 12.),
      overdub: LinearSmooth::new(sample_rate, 12.),
      recycle: LinearSmooth::new(sample_rate, 12.),
      dry: ExponentialSmooth::new(sample_rate, 12.),
      wet: ExponentialSmooth::new(sample_rate, 12.),
      is_initialized: false,
    }
  }

  pub fn set(
    &mut self,
    pitch: f32,
    size: f32,
    scan: f32,
    density: f32,
    stretch: f32,
    time: f32,
    highpass: f32,
    lowpass: f32,
    overdub: f32,
    recycle: f32,
    dry: f32,
    wet: f32,
  ) {
    self.pitch = pitch;
    self.size = size;
    self.scan = scan;
    self.density = density;
    self.stretch = stretch;

    if self.is_initialized {
      self.time.set_target(time);
      self.highpass.set_target(highpass);
      self.lowpass.set_target(lowpass);
      self.overdub.set_target(overdub);
      self.recycle.set_target(recycle);
      self.dry.set_target(dry);
      self.wet.set_target(wet);
    } else {
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
