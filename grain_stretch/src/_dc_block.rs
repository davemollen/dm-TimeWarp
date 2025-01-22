pub struct DcBlock {
  sample_period: f32,
  xm1: (f32, f32),
  ym1: (f32, f32),
}

impl DcBlock {
  pub fn new(sample_rate: f32) -> Self {
    Self {
      sample_period: sample_rate.recip(),
      xm1: (0., 0.),
      ym1: (0., 0.),
    }
  }

  pub fn process(&mut self, x: (f32, f32)) -> (f32, f32) {
    let coeff = 1. - (220.5 * self.sample_period);
    let y = (
      x.0 - self.xm1.0 + coeff * self.ym1.0,
      x.1 - self.xm1.1 + coeff * self.ym1.1,
    );
    self.xm1 = x;
    self.ym1 = y;
    y
  }
}
