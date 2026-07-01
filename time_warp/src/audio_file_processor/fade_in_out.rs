use crate::shared::float_ext::FloatExt;

#[derive(Clone)]
pub struct FadeInOut {
  fade_time_in_samples: usize,
  step_size: f32,
}

impl FadeInOut {
  pub fn new(sample_rate: f32, fade_time: f32) -> Self {
    let fade_time_in_samples = fade_time.mstosamps(sample_rate);

    Self {
      fade_time_in_samples: fade_time_in_samples as usize,
      step_size: fade_time_in_samples.recip(),
    }
  }

  pub fn process(&mut self, samples: &mut [f32]) {
    for i in 0..self.fade_time_in_samples {
      let fade = ((i as f32) * self.step_size).cubic_spline_curve();

      samples[i] *= fade;
      samples[samples.len() - i - 1] *= fade;
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::assert_approximately_eq;

  #[test]
  fn should_apply_fade_in_and_out() {
    let sample_rate = 1000.;
    let mut fade = FadeInOut::new(sample_rate, 10.);
    let mut samples = vec![1.; 1000];

    fade.process(&mut samples);

    for i in 0..10 {
      assert_approximately_eq!(samples[i], (i as f32 * 0.1).cubic_spline_curve(), 7);
      assert_approximately_eq!(
        samples[samples.len() - i - 1],
        (i as f32 * 0.1).cubic_spline_curve(),
        7
      );
    }
  }
}
