use crate::shared::{delta::Delta, phasor::Phasor};

#[derive(Clone)]
pub struct GrainTrigger {
  phasor: Phasor,
  delta: Delta,
}

impl GrainTrigger {
  pub fn new(sample_rate: f32) -> Self {
    Self {
      phasor: Phasor::new(sample_rate),
      delta: Delta::new(),
    }
  }

  pub fn process(&mut self, duration: f32, density: f32, reset: bool) -> bool {
    if reset {
      self.phasor.reset();
      self.delta.reset();
      return true;
    }
    let phase = self.phasor.process(1000. / duration * density);
    self.delta.process(phase) < 0.
  }
}
