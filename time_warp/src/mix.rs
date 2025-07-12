use crate::shared::float_ext::FloatExt;
use std::f32::consts::FRAC_PI_2;

#[derive(Default)]
pub struct Mix {
  mix: f32,
  dry_gain: f32,
  wet_gain: f32,
}

impl Mix {
  pub fn process(&mut self, dry: (f32, f32), wet: (f32, f32), mix: f32) -> (f32, f32) {
    if mix != self.mix {
      let factor = mix * FRAC_PI_2;
      self.mix = mix;
      self.dry_gain = factor.fast_cos_bhaskara();
      self.dry_gain *= self.dry_gain;
      self.wet_gain = factor.fast_sin_bhaskara();
      self.wet_gain *= self.wet_gain;
    }
    (
      dry.0 * self.dry_gain + wet.0 * self.wet_gain,
      dry.1 * self.dry_gain + wet.1 * self.wet_gain,
    )
  }
}
