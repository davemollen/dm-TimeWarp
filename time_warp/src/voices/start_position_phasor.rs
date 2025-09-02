use crate::shared::phasor::Phasor;

pub struct StartPositionPhasor {
  phase_offset: f32,
  offset_phasor: Phasor,
  phasor: Phasor,
}

impl StartPositionPhasor {
  pub fn new(sample_rate: f32) -> Self {
    Self {
      phase_offset: 0.,
      offset_phasor: Phasor::new(sample_rate),
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
    should_reset_offset: bool,
  ) -> f32 {
    // TODO: see if we can move the offset_phasor to params and run this on block-level
    if should_reset_offset {
      self.offset_phasor.reset();
    }
    let record_phase = self.offset_phasor.process(1000. / time);
    if reset_playback {
      self.phase_offset = record_phase;
      self.phasor.reset();
    }

    let is_in_granular_mode = size < 1. || density > 1.;
    if is_in_granular_mode {
      let freq = 1000. / time * (stretch - 1.);
      // TODO: run a phasor per voice so a note trigger starts the sample from the beginning
      (self.phasor.process(freq) + 1. - self.phase_offset).fract()
    } else {
      1. - self.phase_offset
    }
  }

  pub fn reset(&mut self) {
    self.phasor.reset();
    self.offset_phasor.reset();
  }
}
