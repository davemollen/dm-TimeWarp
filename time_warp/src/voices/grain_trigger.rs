use crate::shared::{delta::Delta, phasor::Phasor};

#[derive(Clone)]
pub struct GrainTrigger {
  phasor: Phasor,
  delta: Delta,
}

impl GrainTrigger {
  pub fn new(sample_rate: f32) -> Self {
    Self {
      phasor: Phasor::new(sample_rate as f64),
      delta: Delta::new(),
    }
  }

  pub fn reset(&mut self) {
    self.phasor.reset();
    self.delta.reset();
  }

  pub fn process(&mut self, grain_duration: f64, density: f64, reset: bool) -> bool {
    if reset {
      self.phasor.reset();
      self.delta.reset();
      return true;
    }
    let phase = self.phasor.process(1000. / grain_duration * density);
    self.delta.process(phase) < 0.
  }
}
