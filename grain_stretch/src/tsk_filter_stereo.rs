/* Transposed Sallen Key filter */
mod one_pole_filter_stereo;
use {one_pole_filter_stereo::OnePoleFilterStereo, std::simd::f32x2};

#[derive(Clone, Copy)]
pub enum FilterType {
  Lowpass,
  Highpass,
}

pub struct TSKFilterStereo {
  one_pole_filters: [OnePoleFilterStereo; 2],
  z: f32x2,
}

impl TSKFilterStereo {
  pub fn new(sample_rate: f32) -> Self {
    Self {
      one_pole_filters: [
        OnePoleFilterStereo::new(sample_rate),
        OnePoleFilterStereo::new(sample_rate),
      ],
      z: f32x2::splat(0.),
    }
  }

  pub fn process(
    &mut self,
    input: f32x2,
    freq: f32,
    resonance: f32,
    filter_type: FilterType,
  ) -> f32x2 {
    let y0 = input - self.z;
    let y1 = self.one_pole_filters[0].process(y0, freq, filter_type);
    let y2 = self.one_pole_filters[1].process(y1, freq, filter_type);
    self.z = (y2 - y1) * f32x2::splat(resonance);

    y2
  }
}
