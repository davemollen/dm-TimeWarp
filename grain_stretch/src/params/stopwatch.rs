pub struct Stopwatch {
  count: f32,
  sample_rate: f32,
}

impl Stopwatch {
  pub fn new(sample_rate: f32) -> Self {
    Self {
      count: 0.,
      sample_rate,
    }
  }

  pub fn process(&mut self, start: bool) -> Option<f32> {
    if start {
      self.count += 1.;
      None
    } else if self.count == 0. {
      None
    } else {
      // TODO: make buffer size dynamic
      let buffer_size = 128.;
      Some(self.count / self.sample_rate * buffer_size * 1000.)
    }
  }

  pub fn reset(&mut self) {
    self.count = 0.;
  }
}

#[cfg(test)]
mod tests {
  use super::Stopwatch;

  #[test]
  fn stopwatch() {
    let mut stopwatch = Stopwatch::new(1.);
    assert_eq!(stopwatch.process(true), None);
    assert_eq!(stopwatch.process(true), None);
    assert_eq!(stopwatch.process(true), None);
    assert_eq!(stopwatch.process(false), Some(3000.));
  }
}
