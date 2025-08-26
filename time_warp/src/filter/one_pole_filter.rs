use std::f32::consts::TAU;

pub enum FilterType {
  Lowpass,
  Highpass,
}

pub struct OnePoleFilter {
  t: f32,
  z: f32,
  prev_cutoff_freq: f32,
  b1: f32,
}

impl OnePoleFilter {
  pub fn new(sample_rate: f32) -> Self {
    Self {
      t: sample_rate.recip() * -TAU,
      z: 0.,
      prev_cutoff_freq: 0.,
      b1: 0.,
    }
  }

  pub fn set_cutoff_freq(&mut self, cutoff_freq: f32) {
    if cutoff_freq != self.prev_cutoff_freq {
      self.b1 = (cutoff_freq * self.t).exp();
      self.prev_cutoff_freq = cutoff_freq;
    }
  }

  pub fn process(&mut self, input: f32, filter_type: FilterType) -> f32 {
    match filter_type {
      FilterType::Lowpass => self.apply_filter(input),
      FilterType::Highpass => {
        let filter_output = self.apply_filter(input);
        input - filter_output
      }
    }
  }

  fn apply_filter(&mut self, input: f32) -> f32 {
    let a0 = 1.0 - self.b1;
    self.z = input * a0 + self.z * self.b1;
    self.z
  }
}
