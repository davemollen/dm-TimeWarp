pub trait FloatExt {
  fn dbtoa(self) -> Self;
  fn fast_dbtoa(self) -> Self;
  fn mix(self, right: Self, factor: Self) -> Self;
  fn mstosamps(self, sample_rate: Self) -> Self;
  fn sampstoms(self, sample_rate: Self) -> Self;
  fn cubic_spline_curve(self) -> Self;
}

impl FloatExt for f32 {
  /// Converts decibels to a linear amplitude value
  #[inline(always)]
  fn dbtoa(self) -> Self {
    (10_f32).powf(self * 0.05)
  }

  /// Fast decibels to linear amplitude conversion
  #[inline(always)]
  fn fast_dbtoa(self) -> Self {
    const CONVERSION_FACTOR: f32 = std::f32::consts::LN_10 / 20.0;
    (self * CONVERSION_FACTOR).exp()
  }

  #[inline(always)]
  fn mix(self, right: Self, factor: Self) -> Self {
    self + (right - self) * factor
  }

  /// Convert milliseconds to samples based on the samplerate.
  #[inline(always)]
  fn mstosamps(self, sample_rate: Self) -> Self {
    self * 0.001 * sample_rate
  }

  /// Convert samples to milliseconds based on the samplerate.
  #[inline(always)]
  fn sampstoms(self, sample_rate: Self) -> Self {
    self / sample_rate * 1000.0
  }

  #[inline(always)]
  fn cubic_spline_curve(self) -> Self {
    self * self * (3.0 - 2.0 * self)
  }
}

impl FloatExt for f64 {
  /// Converts decibels to a linear amplitude value
  #[inline(always)]
  fn dbtoa(self) -> Self {
    (10_f64).powf(self * 0.05)
  }

  /// Fast decibels to linear amplitude conversion
  #[inline(always)]
  fn fast_dbtoa(self) -> Self {
    const CONVERSION_FACTOR: f64 = std::f64::consts::LN_10 / 20.0;
    (self * CONVERSION_FACTOR).exp()
  }

  #[inline(always)]
  fn mix(self, right: Self, factor: Self) -> Self {
    self + (right - self) * factor
  }

  /// Convert milliseconds to samples based on the samplerate.
  #[inline(always)]
  fn mstosamps(self, sample_rate: Self) -> Self {
    self * 0.001 * sample_rate
  }

  /// Convert samples to milliseconds based on the samplerate.
  #[inline(always)]
  fn sampstoms(self, sample_rate: Self) -> Self {
    self / sample_rate * 1000.0
  }

  #[inline(always)]
  fn cubic_spline_curve(self) -> Self {
    self * self * (3.0 - 2.0 * self)
  }
}

#[cfg(test)]
mod tests {
  use super::FloatExt;

  #[test]
  fn dbtoa() {
    assert_eq!((-3f32).dbtoa(), 0.70794576);
    assert_eq!((-6f32).dbtoa(), 0.5011872);
    assert_eq!((-12f32).dbtoa(), 0.25118864);
  }

  #[test]
  fn mix() {
    assert_eq!((1f32).mix(0., 0.), 1.);
    assert_eq!((1f32).mix(0., 0.5), 0.5);
    assert_eq!((1f32).mix(0., 1.), 0.);
  }
}
