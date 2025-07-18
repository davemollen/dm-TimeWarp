use std::{f32::consts::PI, mem};

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum Interpolation {
  Step,
  Linear,
  Cosine,
  Cubic,
  Spline,
}

#[derive(Clone)]
pub struct StereoDelayLine {
  buffer: Vec<(f32, f32)>,
  write_pointer: usize,
  sample_rate: f32,
  wrap: usize,
}

impl StereoDelayLine {
  pub fn new(length: usize, sample_rate: f32) -> Self {
    let size = length.next_power_of_two();
    Self {
      buffer: vec![(0.0, 0.0); size],
      write_pointer: 0,
      sample_rate,
      wrap: size - 1,
    }
  }

  pub fn read(&self, time: f32, interp: Interpolation) -> (f32, f32) {
    match interp {
      Interpolation::Step => self.step_interp(time),
      Interpolation::Linear => self.linear_interp(time),
      Interpolation::Cosine => self.cosine_interp(time),
      Interpolation::Cubic => self.cubic_interp(time),
      Interpolation::Spline => self.spline_interp(time),
    }
  }

  pub fn write(&mut self, value: (f32, f32)) {
    self.buffer[self.write_pointer] = value;
    self.write_pointer = self.write_pointer + 1 & self.wrap;
  }

  pub fn set_values(&mut self, mut values: Vec<(f32, f32)>) {
    mem::swap(&mut self.buffer, &mut values);
  }

  pub fn set_write_pointer(&mut self, index: usize) {
    if index >= self.buffer.len() {
      self.write_pointer = 0;
    }
    self.write_pointer = index;
  }

  pub fn get_size(&self) -> usize {
    self.buffer.len()
  }

  fn step_interp(&self, time: f32) -> (f32, f32) {
    let read_pointer =
      (self.write_pointer + self.buffer.len()) as f32 - (self.mstosamps(time) - 0.5).max(1.);
    let index = read_pointer.trunc() as usize;

    self.buffer[index & self.wrap]
  }

  fn linear_interp(&self, time: f32) -> (f32, f32) {
    let read_pointer =
      (self.write_pointer + self.buffer.len()) as f32 - self.mstosamps(time).max(1.);
    let rounded_read_pointer = read_pointer.trunc();
    let mix = read_pointer - rounded_read_pointer;
    let index = rounded_read_pointer as usize;

    let x = self.buffer[index & self.wrap];
    let y = self.buffer[index + 1 & self.wrap];
    (x.0 + (y.0 - x.0) * mix, x.1 + (y.1 - x.1) * mix)
  }

  fn cosine_interp(&self, time: f32) -> (f32, f32) {
    let read_pointer =
      (self.write_pointer + self.buffer.len()) as f32 - self.mstosamps(time).max(1.);
    let rounded_read_pointer = read_pointer.trunc();
    let mix = read_pointer - rounded_read_pointer;
    let index = rounded_read_pointer as usize;

    let cosine_mix = (1. - (mix * PI).cos()) / 2.;
    let x = self.buffer[index & self.wrap];
    let y = self.buffer[index + 1 & self.wrap];
    (
      x.0 + (y.0 - x.0) * cosine_mix,
      x.1 + (y.1 - x.1) * cosine_mix,
    )
  }

  fn cubic_interp(&self, time: f32) -> (f32, f32) {
    let read_pointer =
      (self.write_pointer + self.buffer.len()) as f32 - self.mstosamps(time).max(2.);
    let rounded_read_pointer = read_pointer.trunc();
    let mix = read_pointer - rounded_read_pointer;
    let index = rounded_read_pointer as usize;

    let w = self.buffer[index & self.wrap];
    let x = self.buffer[index + 1 & self.wrap];
    let y = self.buffer[index + 2 & self.wrap];
    let z = self.buffer[index + 3 & self.wrap];

    let a1 = 1. + mix;
    let aa = mix * a1;
    let b = 1. - mix;
    let b1 = 2. - mix;
    let bb = b * b1;
    let fw = -0.1666667 * bb * mix;
    let fx = 0.5 * bb * a1;
    let fy = 0.5 * aa * b1;
    let fz = -0.1666667 * aa * b;
    (
      w.0 * fw + x.0 * fx + y.0 * fy + z.0 * fz,
      w.1 * fw + x.1 * fx + y.1 * fy + z.1 * fz,
    )
  }

  fn spline_interp(&self, time: f32) -> (f32, f32) {
    let read_pointer =
      (self.write_pointer + self.buffer.len()) as f32 - self.mstosamps(time).max(2.);
    let rounded_read_pointer = read_pointer.trunc();
    let mix = read_pointer - rounded_read_pointer;
    let index = rounded_read_pointer as usize;

    let w = self.buffer[index & self.wrap];
    let x = self.buffer[index + 1 & self.wrap];
    let y = self.buffer[index + 2 & self.wrap];
    let z = self.buffer[index + 3 & self.wrap];

    let c0 = x;
    let c1 = (0.5 * (y.0 - w.0), 0.5 * (y.1 - w.1));
    let c2 = (
      w.0 - 2.5 * x.0 + y.0 + y.0 - 0.5 * z.0,
      w.1 - 2.5 * x.1 + y.1 + y.1 - 0.5 * z.1,
    );
    let c3 = (
      0.5 * (z.0 - w.0) + 1.5 * (x.0 - y.0),
      0.5 * (z.1 - w.1) + 1.5 * (x.1 - y.1),
    );
    (
      ((c3.0 * mix + c2.0) * mix + c1.0) * mix + c0.0,
      ((c3.1 * mix + c2.1) * mix + c1.1) * mix + c0.1,
    )
  }

  fn mstosamps(&self, time: f32) -> f32 {
    time * 0.001 * self.sample_rate
  }
}

#[cfg(test)]
mod tests {
  use super::StereoDelayLine;

  #[test]
  fn should_set_values() {
    let mut delay_line = StereoDelayLine::new(2, 1000.);
    let values = vec![(0.4, 0.4), (0.2, 0.2), (-0.2, -0.2), (-0.4, -0.4)];
    delay_line.set_values(values.to_vec());
    assert_eq!(values.len(), 4);
    assert_eq!(delay_line.buffer.len(), 4);
    delay_line
      .buffer
      .iter()
      .zip(values)
      .for_each(|(actual, expected)| {
        assert_eq!(actual.0, expected.0);
        assert_eq!(actual.1, expected.1);
      });
  }
}
