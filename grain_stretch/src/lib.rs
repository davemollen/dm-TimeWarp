#![feature(portable_simd)]
mod params;
mod tsk_filter_stereo;
pub mod shared {
  pub mod float_ext;
  pub mod stereo_delay_line;
}
pub use params::Params;
use {
  crate::shared::stereo_delay_line::{Interpolation, StereoDelayLine},
  params::Smoother,
  tsk_filter_stereo::TSKFilterStereo,
};

pub struct GrainStretch {
  delay_line: StereoDelayLine,
  highpass_filter: TSKFilterStereo,
  lowpass_filter: TSKFilterStereo,
}

impl GrainStretch {
  pub fn new(sample_rate: f32) -> Self {
    Self {
      delay_line: StereoDelayLine::new(sample_rate as usize * 10, sample_rate),
      highpass_filter: TSKFilterStereo::new(sample_rate),
      lowpass_filter: TSKFilterStereo::new(sample_rate),
    }
  }

  pub fn process(&mut self, input: (f32, f32), params: &mut Params) -> (f32, f32) {
    let Params {
      pitch,
      size,
      scan,
      density,
      stretch,
      ..
    } = *params;
    let time = params.time.next();
    let highpass = params.highpass.next();
    let lowpass = params.lowpass.next();
    let overdub = params.overdub.next();
    let recycle = params.recycle.next();
    let dry = params.dry.next();
    let wet = params.wet.next();

    input
  }
}
