use std::{f32::consts::TAU, simd::f32x2};

use super::FilterType;

pub struct OnePoleFilterStereo {
  t: f32,
  z: f32x2,
  prev_cutoff_freq: f32,
  b1: f32x2,
}

impl OnePoleFilterStereo {
  pub fn new(sample_rate: f32) -> Self {
    Self {
      t: sample_rate.recip() * -TAU,
      z: f32x2::splat(0.),
      prev_cutoff_freq: 0.,
      b1: f32x2::splat(0.),
    }
  }

  pub fn process(&mut self, input: f32x2, freq: f32, filter_type: FilterType) -> f32x2 {
    match filter_type {
      FilterType::Lowpass => self.apply_filter(input, freq),
      FilterType::Highpass => {
        let filter_output = self.apply_filter(input, freq);
        input - filter_output
      }
    }
  }

  fn apply_filter(&mut self, input: f32x2, cutoff_freq: f32) -> f32x2 {
    if cutoff_freq != self.prev_cutoff_freq {
      self.b1 = f32x2::splat((cutoff_freq * self.t).exp());
      self.prev_cutoff_freq = cutoff_freq;
    }

    let a0 = f32x2::splat(1.0) - self.b1;
    self.z = input * a0 + self.z * self.b1;
    self.z
  }
}
