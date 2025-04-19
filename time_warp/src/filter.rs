use std::f32::consts::PI;

pub struct Filter {
  z: [(f32, f32); 2],
  double_sample_rate: f32,
  freq_multiplier: f32,
}

impl Filter {
  pub fn new(sample_rate: f32) -> Self {
    let double_sample_rate = sample_rate * 2.;
    Self {
      z: [(0., 0.); 2],
      double_sample_rate,
      freq_multiplier: sample_rate.recip() * PI,
    }
  }

  pub fn process(&mut self, x: (f32, f32), high_pass: f32, low_pass: f32) -> (f32, f32) {
    let w1 = self.convert_to_angular_frequency(high_pass);
    let w2 = self.convert_to_angular_frequency(low_pass);

    let z_domain_coeffs = self.get_z_domain_coefficients(w1, w2);

    self.apply_filter(x, z_domain_coeffs)
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

  fn apply_filter(&mut self, x: (f32, f32), (b, a): ([f32; 3], [f32; 3])) -> (f32, f32) {
    let y = (x.0 * b[0] + self.z[0].0, x.1 * b[0] + self.z[0].1);
    self.z[0] = (self.z[1].0 - a[1] * y.0, self.z[1].1 - a[1] * y.1);
    self.z[1] = (x.0 * b[2] - a[2] * y.0, x.1 * b[2] - a[2] * y.1);

    return y;
  }
}
