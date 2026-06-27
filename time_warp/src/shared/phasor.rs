#[derive(Clone)]
pub struct Phasor {
  sample_period: f64,
  x: f64,
}

impl Phasor {
  pub fn new(sample_rate: f64) -> Self {
    Self {
      sample_period: sample_rate.recip(),
      x: 0.,
    }
  }

  pub fn process(&mut self, freq: f64) -> f64 {
    let y = self.x;
    self.x = self.wrap(self.x + freq * self.sample_period);
    y
  }

  pub fn reset(&mut self) {
    self.x = 0.;
  }

  fn wrap(&self, input: f64) -> f64 {
    if input >= 1. {
      input - 1.
    } else if input < 0. {
      input + 1.
    } else {
      input
    }
  }
}
