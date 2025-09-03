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

  pub fn process(&mut self, freq: f32, speed: f32, stretch: f32, is_in_granular_mode: bool) -> f32 {
    let freq = if is_in_granular_mode {
      freq * (stretch - 1.)
    } else {
      freq * ((speed * stretch.signum()) - 1.)
    };
    (self.phasor.process(freq) + self.offset).fract()
  }

  pub fn reset(&mut self, offset: f32) {
    self.phasor.reset();
    self.offset = offset;
  }
}
