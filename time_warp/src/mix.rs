use crate::shared::float_ext::FloatExt;

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
      let wet_gain = mix.cubic_spline_curve();
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
  use crate::assert_approximately_eq;

  #[test]
  fn should_pass_dry() {
    let mut mix = Mix::new();
    assert_approximately_eq!(mix.process(0.8, -0.4, 0.), 0.8, 6);
    assert_approximately_eq!(mix.process(0.8, -0.4, 1.), -0.4, 6);
  }
}
