pub trait TupleExt {
  fn add(self, right: Self) -> Self;
  fn multiply(self, right: f32) -> Self;
}

impl TupleExt for (f32, f32) {
  fn add(self, right: Self) -> Self {
    (self.0 + right.0, self.1 + right.1)
  }

  fn multiply(self, right: f32) -> Self {
    (self.0 * right, self.1 * right)
  }
}
