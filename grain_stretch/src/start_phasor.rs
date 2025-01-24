use crate::shared::phasor::Phasor;

pub struct StartPhasor {
  phasor: Phasor,
  prev_speed: f32,
  sample_rate: f32,
}

impl StartPhasor {
  pub fn new(sample_rate: f32) -> Self {
    Self {
      phasor: Phasor::new(sample_rate),
      prev_speed: 1.,
      sample_rate,
    }
  }

  pub fn process(&mut self, speed: f32, time: f32, size: f32, density: f32, stretch: f32) -> f32 {
    let freq = if size > 0. || density > 0. {
      self.sample_rate / time * (stretch * speed.signum() - 1.)
    } else {
      self.maybe_reset_phasor(speed);
      self.sample_rate / time * (speed - 1.)
    };

    self.phasor.process(freq)
  }

  fn maybe_reset_phasor(&mut self, speed: f32) {
    if speed != self.prev_speed && speed == 1. {
      self.phasor.reset();
    }
    self.prev_speed = speed;
  }
}
