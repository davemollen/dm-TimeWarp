pub struct Stopwatch {
  sample_count: usize,
  sample_rate: f32,
}

impl Stopwatch {
  pub fn new(sample_rate: f32) -> Self {
    Self {
      sample_count: 0,
      sample_rate,
    }
  }

  pub fn process(&mut self, start: bool, buffer_size: usize) -> Option<f32> {
    if start {
      self.sample_count += buffer_size;
      None
    } else if self.sample_count == 0 {
      None
    } else {
      Some(self.sample_count as f32 / self.sample_rate * 1000.)
    }
  }

  pub fn reset(&mut self) {
    self.sample_count = 0;
  }
}

#[cfg(test)]
mod tests {
  use super::Stopwatch;

  #[test]
  fn stopwatch() {
    let mut stopwatch = Stopwatch::new(1.);
    assert_eq!(stopwatch.process(true, 1), None);
    assert_eq!(stopwatch.process(true, 1), None);
    assert_eq!(stopwatch.process(true, 1), None);
    assert_eq!(stopwatch.process(false, 1), Some(3000.));
  }
}
