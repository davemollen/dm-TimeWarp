use crate::{
  notes::{Note, NoteState},
  shared::float_ext::FloatExt,
};

#[derive(PartialEq, Clone)]
pub enum ADSRStage {
  Attack,
  Decay,
  Sustain,
  Release,
  Retrigger,
  Idle,
}

#[derive(Clone)]
pub struct ADSR {
  x: f32,
  sample_rate: f32,
  retrigger_step_size: f32,
  stage: ADSRStage,
  speed: f32,
}

impl ADSR {
  pub fn new(sample_rate: f32) -> Self {
    let retrigger_step_size = 5_f32.mstosamps(sample_rate).recip();
    Self {
      x: 0.,
      sample_rate,
      retrigger_step_size,
      stage: ADSRStage::Idle,
      speed: 1.,
    }
  }

  pub fn process(
    &mut self,
    note: &mut Note,
    attack_time: f32,
    decay_time: f32,
    sustain: f32,
    release_time: f32,
  ) -> f32 {
    match note.get_state() {
      NoteState::On => self.stage = ADSRStage::Attack,
      NoteState::Off => self.stage = ADSRStage::Release,
      NoteState::Stolen => self.stage = ADSRStage::Retrigger,
      _ => (),
    }

    match self.stage {
      ADSRStage::Idle => {
        self.x = 0.;
      }
      ADSRStage::Attack => {
        self.speed = note.get_speed();
        let attack_step_size = attack_time.mstosamps(self.sample_rate).recip();
        let next_x = self.x + attack_step_size;
        if next_x >= 1. {
          self.x = 1.;
          self.stage = ADSRStage::Decay;
        } else {
          self.x = next_x;
        }
      }
      ADSRStage::Decay => {
        let decay_step_size =
          decay_time.mstosamps(self.sample_rate).recip() * (1. - sustain).recip();
        let next_x = self.x - decay_step_size;
        if next_x <= sustain {
          self.x = sustain;
          self.stage = ADSRStage::Sustain;
        } else {
          self.x = next_x;
        }
      }
      ADSRStage::Sustain => {
        self.x = sustain;
      }
      ADSRStage::Release => {
        let release_step_size =
          release_time.mstosamps(self.sample_rate).recip() * (sustain).recip();
        let next_x = self.x - release_step_size;
        if next_x <= 0. {
          self.x = 0.;
          note.set_state(NoteState::Idle);
        } else {
          self.x = next_x;
        }
      }
      ADSRStage::Retrigger => {
        let next_x = self.x - self.retrigger_step_size;
        if next_x <= 0. {
          self.x = 0.;
          self.stage = ADSRStage::Attack;
        } else {
          self.x = next_x;
        }
      }
    };

    self.x * note.get_gain()
  }

  pub fn get_speed(&self) -> f32 {
    self.speed
  }
}
