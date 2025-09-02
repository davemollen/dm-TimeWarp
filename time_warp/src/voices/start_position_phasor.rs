use crate::shared::phasor::Phasor;

pub struct StartPositionPhasor {
  phasor: Phasor,
}

impl StartPositionPhasor {
  pub fn new(sample_rate: f32) -> Self {
    Self {
      phasor: Phasor::new(sample_rate),
    }
  }

  pub fn process(
    &mut self,
    time: f32,
    size: f32,
    density: f32,
    stretch: f32,
    reset_playback: bool,
    phase_offset: f32,
  ) -> f32 {
    if reset_playback {
      self.phasor.reset();
    }

    let is_in_granular_mode = size < 1. || density > 1.;
    if is_in_granular_mode {
      let freq = 1000. / time * (stretch - 1.);
      // TODO: run a phasor per voice so a note trigger starts the sample from the beginning
      (self.phasor.process(freq) + 1. - phase_offset).fract()
    } else {
      1. - phase_offset
    }
  }
}
