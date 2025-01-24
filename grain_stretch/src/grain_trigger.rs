use crate::shared::{delta::Delta, float_ext::FloatExt, phasor::Phasor};

pub struct GrainTrigger {
  phasor: Phasor,
  delta: Delta,
  sample_rate: f32,
}

impl GrainTrigger {
  pub fn new(sample_rate: f32) -> Self {
    Self {
      phasor: Phasor::new(sample_rate),
      delta: Delta::new(),
      sample_rate,
    }
  }

  pub fn process(&mut self, duration: f32, density: f32) -> bool {
    let phase = self.phasor.process(self.sample_rate / duration * density);
    self.delta.process(phase) < 0.
  }
}
