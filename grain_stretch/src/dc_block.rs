pub struct DcBlock {
  coeff: f32,
  xm1: (f32, f32),
  ym1: (f32, f32),
}

impl DcBlock {
  pub fn new(sample_rate: f32) -> Self {
    Self {
      coeff: 1. - (220.5 * sample_rate.recip()),
      xm1: (0., 0.),
      ym1: (0., 0.),
    }
  }

  pub fn process(&mut self, x: (f32, f32)) -> (f32, f32) {
    if self.check_subnormals(x) {
      self.xm1 = x;
      return x;
    }

    let y = (
      x.0 - self.xm1.0 + self.coeff * self.ym1.0,
      x.1 - self.xm1.1 + self.coeff * self.ym1.1,
    );
    self.xm1 = x;
    self.ym1 = y;
    y
  }

  fn check_subnormals(&self, x: (f32, f32)) -> bool {
    (x.0 - self.xm1.0).abs() <= f32::EPSILON && (x.1 - self.xm1.1).abs() <= f32::EPSILON
  }
}
