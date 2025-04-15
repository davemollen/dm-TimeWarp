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

  pub fn process(
    &mut self,
    speed: f32,
    time: f32,
    size: f32,
    density: f32,
    stretch: f32,
    is_recording: bool,
  ) -> f32 {
    let recording_speed = if is_recording { 1. } else { 0. };
    let freq = if size > 0. || density > 0. {
      1000. / time * (stretch * speed.signum() - recording_speed)
    } else {
      if speed != self.prev_speed && speed == 1. {
        self.phasor.reset();
      }
      self.prev_speed = speed;
      1000. / time * (speed - recording_speed)
    };

    self.phasor.process(freq)
  }
}
