use crate::shared::float_ext::FloatExt;
use std::f32::consts::FRAC_PI_2;

pub struct Mix;

impl Mix {
  pub fn process(dry: (f32, f32), wet: (f32, f32), mix: f32) -> (f32, f32) {
    let factor = mix * FRAC_PI_2;
    let dry_gain = factor.fast_cos();
    let wet_gain = factor.fast_sin();
    let dry_left = dry.0 * dry_gain;
    let dry_right = dry.1 * dry_gain;
    let wet_left = wet.0 * wet_gain;
    let wet_right = wet.1 * wet_gain;
    (dry_left + wet_left, dry_right + wet_right)
  }
}

#[cfg(test)]
mod tests {
  use super::Mix;

  fn assert_approximately_eq(left: f32, right: f32) {
    assert_eq!((left * 100.).round() / 100., (right * 100.).round() / 100.)
  }

  #[test]
  fn mix() {
    let first = Mix::process((0., 0.), (1., 1.), 0.);
    let second = Mix::process((0., 0.), (1., 1.), 0.5);
    let third = Mix::process((0., 0.), (1., 1.), 1.);
    assert_approximately_eq(first.0, 0.);
    assert_approximately_eq(first.1, 0.);
    assert_approximately_eq(second.0, 0.707);
    assert_approximately_eq(second.1, 0.707);
    assert_approximately_eq(third.0, 1.);
    assert_approximately_eq(third.1, 1.);
  }
}
