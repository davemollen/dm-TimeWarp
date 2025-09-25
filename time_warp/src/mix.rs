use crate::shared::float_ext::FloatExt;
use std::f32::consts::FRAC_PI_2;

pub struct Mix {
  mix: f32,
  dry_gain: f32,
  wet_gain: f32,
}

impl Mix {
  pub fn new() -> Self {
    Self {
      mix: 0.,
      dry_gain: 1.,
      wet_gain: 0.,
    }
  }

  pub fn process(&mut self, dry: f32, wet: f32, mix: f32) -> f32 {
    if mix != self.mix {
      let factor = mix * FRAC_PI_2;
      let wet_gain = factor.fast_sin_bhaskara();
      self.wet_gain = wet_gain * wet_gain;
      self.dry_gain = 1. - self.wet_gain;
      self.mix = mix;
    }
    dry * self.dry_gain + wet * self.wet_gain
  }
}

#[cfg(test)]
mod tests {
  use super::Mix;

  fn assert_approximately_eq(left: f32, right: f32, digits: usize) {
    let tol = 10f32.powi(-(digits as i32));
    let diff = (left - right).abs();
    assert!(
      diff <= tol,
      "Values are not approximately equal: left={left}, right={right}, diff={diff}, tol={tol}"
    );
  }

  #[test]
  fn should_pass_dry() {
    let mut mix = Mix::new();
    assert_approximately_eq(mix.process(0.8, -0.4, 0.), 0.8, 6);
    assert_approximately_eq(mix.process(0.8, -0.4, 1.), -0.4, 6);
  }
}
