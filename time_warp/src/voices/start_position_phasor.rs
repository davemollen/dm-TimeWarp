use crate::shared::phasor::Phasor;

pub struct StartPositionPhasor {
  phasor: Phasor,
  prev_is_in_granular_mode: bool,
}

impl StartPositionPhasor {
  pub fn new(sample_rate: f32) -> Self {
    Self {
      phasor: Phasor::new(sample_rate),
      prev_is_in_granular_mode: false,
    }
  }

  pub fn process(&mut self, time: f32, size: f32, density: f32, stretch: f32) -> f32 {
    let is_in_granular_mode = size < 1. || density > 1.;
    if is_in_granular_mode {
      let freq = 1000. / time * (stretch - 1.);
      return self.phasor.process(freq);
    } else if self.prev_is_in_granular_mode {
      self.phasor.reset();
    }
    self.prev_is_in_granular_mode = is_in_granular_mode;
    return 0.;
  }

  pub fn reset(&mut self) {
    self.phasor.reset();
  }
}
