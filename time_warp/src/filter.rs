use std::f32::consts::PI;

pub struct Filter {
  coefficients: ([f32; 3], [f32; 3]),
  z: [f32; 2],
  double_sample_rate: f32,
  freq_multiplier: f32,
}

impl Filter {
  pub fn new(sample_rate: f32) -> Self {
    let double_sample_rate = sample_rate * 2.;
    Self {
      coefficients: ([0.; 3], [0.; 3]),
      z: [0.; 2],
      double_sample_rate,
      freq_multiplier: sample_rate.recip() * PI,
    }
  }

  pub fn set_coefficients(&mut self, highpass: f32, lowpass: f32) {
    let w1 = self.convert_to_angular_frequency(highpass);
    let w2 = self.convert_to_angular_frequency(lowpass);
    self.coefficients = self.get_z_domain_coefficients(w1, w2);
  }

  pub fn process(&mut self, x: f32) -> f32 {
    let (b, a) = self.coefficients;
    let y = x * b[0] + self.z[0];
    self.z[0] = self.z[1] - a[1] * y;
    self.z[1] = x * b[2] - a[2] * y;

    y
  }

  fn convert_to_angular_frequency(&self, freq: f32) -> f32 {
    (freq * self.freq_multiplier).tan() * self.double_sample_rate
  }

  fn get_z_domain_coefficients(&self, w1: f32, w2: f32) -> ([f32; 3], [f32; 3]) {
    let a = w1 - self.double_sample_rate;
    let b = w1 + self.double_sample_rate;
    let c = w2 - self.double_sample_rate;
    let d = w2 + self.double_sample_rate;

    let a0 = b * d;
    let a1 = (a * d + b * c) / a0;
    let a2 = (a * c) / a0;
    let b2 = (w2 * self.double_sample_rate) / a0;
    let b0 = b2 * -1.;

    ([b0, 0., b2], [a0, a1, a2])
  }
}
