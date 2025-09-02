use crate::shared::phasor::Phasor;

#[derive(Clone)]
pub struct StartPositionPhasor {
  phasor: Phasor,
  offset: f32,
}

impl StartPositionPhasor {
  pub fn new(sample_rate: f32) -> Self {
    Self {
      phasor: Phasor::new(sample_rate),
      offset: 0.,
    }
  }

  pub fn process(&mut self, is_in_granular_mode: bool, freq: f32) -> f32 {
    if is_in_granular_mode {
      (self.phasor.process(freq) + self.offset).fract()
    } else {
      self.offset
    }
  }

  pub fn reset(&mut self, offset: f32) {
    self.phasor.reset();
    self.offset = offset;
  }
}
