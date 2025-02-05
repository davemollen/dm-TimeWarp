enum Stage {
  Attack,
  Decay,
  Release,
}

pub struct ADSR {
  x: f32,
}

impl ADSR {
  pub fn new(sample_rate: f32) -> Self {}

  pub fn process(
    &mut self,
    gain: f32,
    attack_time: f32,
    decay_time: f32,
    sustain: f32,
    release_time: f32,
  ) -> f32 {
  }
}
