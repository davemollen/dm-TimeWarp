use crate::shared::phasor::Phasor;

#[derive(Clone)]
pub struct StartPhasor {
  phasor: Phasor,
  prev_speed: f32,
}

impl StartPhasor {
  pub fn new(sample_rate: f32) -> Self {
    Self {
      phasor: Phasor::new(sample_rate),
      prev_speed: 1.,
    }
  }

  pub fn process(&mut self, speed: f32, time: f32, size: f32, density: f32, stretch: f32) -> f32 {
    let freq = if size < 1. || density > 0. {
      1000. / time * (stretch - 1.)
    } else {
      if speed != self.prev_speed && speed == 1. {
        self.phasor.reset();
      }
      self.prev_speed = speed;
      1000. / time * (speed - 1.)
    };

    self.phasor.process(freq)
  }

  pub fn reset(&mut self) {
    self.phasor.reset();
  }
}
