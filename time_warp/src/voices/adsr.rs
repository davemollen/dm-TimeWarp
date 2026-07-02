use crate::{
  notes::{ADSRStage, Note},
  shared::float_ext::FloatExt,
};

#[derive(Clone)]
pub struct ADSR {
  x: f32,
  adsr_output: f32,
  sample_rate: f32,
  retrigger_step_size: f32,
  gain: f32,
  speed: f64,
  trigger: bool,
  ramp_down_start_value: Option<f32>,
}

impl ADSR {
  pub fn new(sample_rate: f32, retrigger_time: f32) -> Self {
    let retrigger_step_size = retrigger_time.mstosamps(sample_rate).recip();
    Self {
      adsr_output: 0.,
      x: 0.,
      sample_rate,
      retrigger_step_size,
      gain: 1.,
      speed: 1.,
      trigger: false,
      ramp_down_start_value: None,
    }
  }

  pub fn reset(&mut self) {
    self.x = 0.;
    self.gain = 1.;
    self.speed = 1.;
    self.trigger = false;
    self.ramp_down_start_value = None;
  }

  pub fn process(
    &mut self,
    note: &mut Note,
    attack_time: f32,
    decay_time: f32,
    sustain: f32,
    release_time: f32,
  ) -> f32 {
    let adsr_stage = note.get_adsr_stage().clone();
    match adsr_stage {
      ADSRStage::Idle => {
        self.x = 0.;
        self.adsr_output = 0.;
      }
      ADSRStage::Attack => {
        self.trigger = self.x == 0.;
        self.gain = note.get_gain();
        self.speed = note.get_speed();
        let attack_step_size = attack_time.mstosamps(self.sample_rate).recip();
        let next_x = self.x + attack_step_size;
        if next_x >= 1. {
          self.x = 1.;
          note.set_adsr_stage(ADSRStage::Decay);
        } else {
          self.x = next_x;
        }

        self.adsr_output = self.x;
      }
      ADSRStage::Decay => {
        if sustain == 1. {
          note.set_adsr_stage(ADSRStage::Sustain);
        } else {
          let decay_step_size = decay_time.mstosamps(self.sample_rate).recip();
          let next_x = self.x - decay_step_size;
          if next_x <= 0. {
            self.x = 1.;
            note.set_adsr_stage(ADSRStage::Sustain);
          } else {
            self.x = next_x;
          }
        }

        self.adsr_output = self.x.cube() * (1. - sustain) + sustain;
      }
      ADSRStage::Sustain => {
        self.adsr_output = sustain;
      }
      ADSRStage::Release => {
        let range = match self.ramp_down_start_value {
          Some(range) => range,
          None => {
            self.ramp_down_start_value = Some(self.adsr_output);
            self.x = 1.;
            self.adsr_output
          }
        };

        let release_step_size = release_time.mstosamps(self.sample_rate).recip();
        let next_x = self.x - release_step_size;
        if next_x <= 0. {
          self.x = 0.;
          self.ramp_down_start_value = None;
          note.set_adsr_stage(ADSRStage::Idle);
        } else {
          self.x = next_x;
        }

        self.adsr_output = self.x.cube() * range;
      }
      ADSRStage::Retrigger => {
        let range = match self.ramp_down_start_value {
          Some(range) => range,
          None => {
            self.ramp_down_start_value = Some(self.adsr_output);
            self.x = 1.;
            self.adsr_output
          }
        };

        self.x = self.adsr_output;
        let next_x = self.x - self.retrigger_step_size;
        if next_x <= 0. {
          self.x = 0.;
          self.ramp_down_start_value = None;
          note.set_adsr_stage(ADSRStage::Attack);
        } else {
          self.x = next_x;
        }

        self.adsr_output = self.x.cube() * range;
      }
    };

    self.adsr_output * self.gain
  }

  pub fn get_speed(&self) -> f64 {
    self.speed
  }

  pub fn get_trigger(&self) -> bool {
    self.trigger
  }
}

#[cfg(test)]
mod tests {
  use super::ADSR;
  use crate::{
    assert_approximately_eq,
    notes::{ADSRStage, Note},
  };

  #[test]
  fn regular_adsr() {
    let mut note = Note::default();
    let mut adsr = ADSR::new(10., 1000.);

    assert!(*note.get_adsr_stage() == ADSRStage::Idle);
    note.note_on(60, 1.);
    // attack stage
    assert!(*note.get_adsr_stage() == ADSRStage::Attack);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approximately_eq!(ramp, 0.1, 6);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approximately_eq!(ramp, 0.2, 6);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approximately_eq!(ramp, 0.3, 6);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approximately_eq!(ramp, 0.4, 6);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approximately_eq!(ramp, 0.5, 6);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approximately_eq!(ramp, 0.6, 6);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approximately_eq!(ramp, 0.7, 6);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approximately_eq!(ramp, 0.8, 6);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approximately_eq!(ramp, 0.9, 6);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approximately_eq!(ramp, 1.0, 6);
    // decay stage
    assert!(*note.get_adsr_stage() == ADSRStage::Decay);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approximately_eq!(ramp, 0.9, 6);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approximately_eq!(ramp, 0.8, 6);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approximately_eq!(ramp, 0.7, 6);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approximately_eq!(ramp, 0.6, 6);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approximately_eq!(ramp, 0.5, 6);
    // sustain stage
    assert!(*note.get_adsr_stage() == ADSRStage::Sustain);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approximately_eq!(ramp, 0.5, 6);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approximately_eq!(ramp, 0.5, 6);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approximately_eq!(ramp, 0.5, 6);
    // release stage
    note.note_off();
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approximately_eq!(ramp, 0.4, 6);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approximately_eq!(ramp, 0.3, 6);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approximately_eq!(ramp, 0.2, 6);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approximately_eq!(ramp, 0.1, 6);
    assert!(*note.get_adsr_stage() == ADSRStage::Release);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approximately_eq!(ramp, 0., 6);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approximately_eq!(ramp, 0., 6);
    assert!(*note.get_adsr_stage() == ADSRStage::Idle);
  }

  #[test]
  fn should_apply_gain_based_on_velocity() {
    let mut note = Note::default();
    let mut adsr = ADSR::new(10., 1000.);
    assert!(*note.get_adsr_stage() == ADSRStage::Idle);
    note.note_on(60, 0.5);
    let ramp = adsr.process(&mut note, 0., 0., 1., 0.);
    assert_eq!(ramp, 0.70710677);
    let ramp = adsr.process(&mut note, 0., 0., 0.5, 0.);
    assert_eq!(ramp, 0.35355338);
  }

  #[test]
  fn retrigger_adsr_during_attack() {
    let mut note = Note::default();
    let mut adsr = ADSR::new(10., 1000.);

    assert!(*note.get_adsr_stage() == ADSRStage::Idle);
    note.note_on(60, 1.);
    // attack stage
    assert!(*note.get_adsr_stage() == ADSRStage::Attack);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approximately_eq!(ramp, 0.1, 6);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approximately_eq!(ramp, 0.2, 6);
    // retrigger stage
    note.steal_note(64, 1.);
    assert!(*note.get_adsr_stage() == ADSRStage::Retrigger);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approximately_eq!(ramp, 0.1, 6);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approximately_eq!(ramp, 0., 6);
    // attack stage
    assert!(*note.get_adsr_stage() == ADSRStage::Attack);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approximately_eq!(ramp, 0.1, 6);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approximately_eq!(ramp, 0.2, 6);
  }

  #[test]
  fn retrigger_adsr_during_decay() {
    let mut note = Note::default();
    let mut adsr = ADSR::new(10., 1000.);

    assert!(*note.get_adsr_stage() == ADSRStage::Idle);
    note.note_on(60, 1.);
    // attack stage
    assert!(*note.get_adsr_stage() == ADSRStage::Attack);
    let ramp = adsr.process(&mut note, 100., 500., 0.5, 500.);
    assert_approximately_eq!(ramp, 1., 6);
    // decay stage
    assert!(*note.get_adsr_stage() == ADSRStage::Decay);
    let ramp = adsr.process(&mut note, 100., 500., 0.5, 500.);
    assert_approximately_eq!(ramp, 0.9, 6);
    let ramp = adsr.process(&mut note, 100., 500., 0.5, 500.);
    assert_approximately_eq!(ramp, 0.8, 6);
    // retrigger stage
    note.steal_note(64, 1.);
    assert!(*note.get_adsr_stage() == ADSRStage::Retrigger);
    let ramp = adsr.process(&mut note, 100., 500., 0.5, 500.);
    assert_approximately_eq!(ramp, 0.7, 6);
    let ramp = adsr.process(&mut note, 100., 500., 0.5, 500.);
    assert_approximately_eq!(ramp, 0.6, 6);
    let ramp = adsr.process(&mut note, 100., 500., 0.5, 500.);
    assert_approximately_eq!(ramp, 0.5, 6);
    let ramp = adsr.process(&mut note, 100., 500., 0.5, 500.);
    assert_approximately_eq!(ramp, 0.4, 6);
    let ramp = adsr.process(&mut note, 100., 500., 0.5, 500.);
    assert_approximately_eq!(ramp, 0.3, 6);
    let ramp = adsr.process(&mut note, 100., 500., 0.5, 500.);
    assert_approximately_eq!(ramp, 0.2, 6);
    let ramp = adsr.process(&mut note, 100., 500., 0.5, 500.);
    assert_approximately_eq!(ramp, 0.1, 6);
    assert!(*note.get_adsr_stage() == ADSRStage::Retrigger);
    let ramp = adsr.process(&mut note, 100., 500., 0.5, 500.);
    // attack stage
    assert!(*note.get_adsr_stage() == ADSRStage::Attack);
    assert_approximately_eq!(ramp, 0., 6);
    let ramp = adsr.process(&mut note, 100., 500., 0.5, 500.);
    assert_approximately_eq!(ramp, 1., 6);
    // decay stage
    assert!(*note.get_adsr_stage() == ADSRStage::Decay);
    let ramp = adsr.process(&mut note, 100., 500., 0.5, 500.);
    assert_approximately_eq!(ramp, 0.9, 6);
  }

  #[test]
  fn retrigger_adsr_during_sustain() {
    let mut note = Note::default();
    let mut adsr = ADSR::new(10., 1000.);

    assert!(*note.get_adsr_stage() == ADSRStage::Idle);
    note.note_on(60, 1.);
    // attack stage
    assert!(*note.get_adsr_stage() == ADSRStage::Attack);
    let ramp = adsr.process(&mut note, 100., 100., 0.5, 500.);
    assert_approximately_eq!(ramp, 1., 6);
    // decay stage
    assert!(*note.get_adsr_stage() == ADSRStage::Decay);
    let ramp = adsr.process(&mut note, 100., 100., 0.5, 500.);
    assert_approximately_eq!(ramp, 0.5, 6);
    // sustain stage
    assert!(*note.get_adsr_stage() == ADSRStage::Sustain);
    let ramp = adsr.process(&mut note, 100., 100., 0.5, 500.);
    assert_approximately_eq!(ramp, 0.5, 6);
    let ramp = adsr.process(&mut note, 100., 100., 0.5, 500.);
    assert_approximately_eq!(ramp, 0.5, 6);
    // retrigger stage
    note.steal_note(64, 1.);
    assert!(*note.get_adsr_stage() == ADSRStage::Retrigger);
    let ramp = adsr.process(&mut note, 100., 100., 0.5, 500.);
    assert_approximately_eq!(ramp, 0.4, 6);
    let ramp = adsr.process(&mut note, 100., 100., 0.5, 500.);
    assert_approximately_eq!(ramp, 0.3, 6);
    let ramp = adsr.process(&mut note, 100., 100., 0.5, 500.);
    assert_approximately_eq!(ramp, 0.2, 6);
    let ramp = adsr.process(&mut note, 100., 100., 0.5, 500.);
    assert_approximately_eq!(ramp, 0.1, 6);
    let ramp = adsr.process(&mut note, 100., 100., 0.5, 500.);
    assert_approximately_eq!(ramp, 0., 6);
    let ramp = adsr.process(&mut note, 100., 100., 0.5, 500.);
    assert_approximately_eq!(ramp, 0., 6);
    // attack stage
    assert!(*note.get_adsr_stage() == ADSRStage::Attack);
    let ramp = adsr.process(&mut note, 100., 100., 0.5, 500.);
    assert_approximately_eq!(ramp, 1., 6);
  }

  #[test]
  fn retrigger_adsr_during_release() {
    let mut note = Note::default();
    let mut adsr = ADSR::new(10., 1000.);

    assert!(*note.get_adsr_stage() == ADSRStage::Idle);
    note.note_on(60, 1.);
    // attack stage
    assert!(*note.get_adsr_stage() == ADSRStage::Attack);
    let ramp = adsr.process(&mut note, 100., 100., 0.5, 1000.);
    assert_approximately_eq!(ramp, 1., 6);
    // decay stage
    assert!(*note.get_adsr_stage() == ADSRStage::Decay);
    let ramp = adsr.process(&mut note, 100., 100., 0.5, 1000.);
    assert_approximately_eq!(ramp, 0.5, 6);
    // sustain stage
    assert!(*note.get_adsr_stage() == ADSRStage::Sustain);
    let ramp = adsr.process(&mut note, 100., 100., 0.5, 1000.);
    assert_approximately_eq!(ramp, 0.5, 6);
    // release stage
    note.note_off();
    assert!(*note.get_adsr_stage() == ADSRStage::Release);
    let ramp = adsr.process(&mut note, 100., 100., 0.5, 1000.);
    assert_approximately_eq!(ramp, 0.45, 6);
    assert!(*note.get_adsr_stage() == ADSRStage::Release);
    let ramp = adsr.process(&mut note, 100., 100., 0.5, 1000.);
    assert_approximately_eq!(ramp, 0.4, 6);
    // retrigger stage
    note.steal_note(64, 1.);
    assert!(*note.get_adsr_stage() == ADSRStage::Retrigger);
    let ramp = adsr.process(&mut note, 100., 100., 0.5, 1000.);
    assert_approximately_eq!(ramp, 0.3, 6);
    let ramp = adsr.process(&mut note, 100., 100., 0.5, 1000.);
    assert_approximately_eq!(ramp, 0.2, 6);
    let ramp = adsr.process(&mut note, 100., 100., 0.5, 1000.);
    assert_approximately_eq!(ramp, 0.1, 6);
    let ramp = adsr.process(&mut note, 100., 100., 0.5, 1000.);
    assert_approximately_eq!(ramp, 0., 6);

    // attack stage
    assert!(*note.get_adsr_stage() == ADSRStage::Attack);
    let ramp = adsr.process(&mut note, 100., 100., 0.5, 1000.);
    assert_approximately_eq!(ramp, 1., 6);
  }

  #[test]
  fn holds_gain_and_speed_until_retrigger_is_finished() {
    let mut note = Note::default();
    let mut adsr = ADSR::new(10., 1000.);

    assert!(*note.get_adsr_stage() == ADSRStage::Idle);
    // attack stage
    note.note_on(72, 0.75);
    assert!(*note.get_adsr_stage() == ADSRStage::Attack);
    assert_eq!(adsr.get_speed(), 1.);
    let ramp = adsr.process(&mut note, 100., 1000., 0.5, 1000.);
    assert_eq!(adsr.get_speed(), 2.);
    assert_approximately_eq!(ramp, 0.8660254, 6);
    // decay stage
    assert!(*note.get_adsr_stage() == ADSRStage::Decay);
    let ramp = adsr.process(&mut note, 100., 500., 0.5, 1000.);
    assert_approximately_eq!(ramp, 0.7794228, 6);
    let ramp = adsr.process(&mut note, 100., 500., 0.5, 1000.);
    assert_approximately_eq!(ramp, 0.69282025, 6);
    // retrigger stage
    note.steal_note(48, 1.);
    assert!(*note.get_adsr_stage() == ADSRStage::Retrigger);
    assert_eq!(adsr.get_speed(), 2.);
    assert_eq!(adsr.gain, 0.8660254);
    let ramp = adsr.process(&mut note, 100., 500., 0.5, 1000.);
    assert_eq!(adsr.get_speed(), 2.);
    assert_eq!(adsr.gain, 0.8660254);
    assert_approximately_eq!(ramp, 0.6062177, 6);
    let ramp = adsr.process(&mut note, 100., 500., 0.5, 1000.);
    assert_approximately_eq!(ramp, 0.5196152, 6);
    let ramp = adsr.process(&mut note, 100., 500., 0.5, 1000.);
    assert_approximately_eq!(ramp, 0.4330126, 6);
    let ramp = adsr.process(&mut note, 100., 500., 0.5, 1000.);
    assert_approximately_eq!(ramp, 0.3464101, 6);
    let ramp = adsr.process(&mut note, 100., 500., 0.5, 1000.);
    assert_approximately_eq!(ramp, 0.25980756, 6);
    let ramp = adsr.process(&mut note, 100., 500., 0.5, 1000.);
    assert_approximately_eq!(ramp, 0.17320502, 6);
    let ramp = adsr.process(&mut note, 100., 500., 0.5, 1000.);
    assert_approximately_eq!(ramp, 0.08660248, 6);
    assert_eq!(adsr.get_speed(), 2.);
    assert_eq!(adsr.gain, 0.8660254);
    let ramp = adsr.process(&mut note, 100., 500., 0.5, 1000.);
    assert_approximately_eq!(ramp, 0., 6);
    // attack stage
    assert!(*note.get_adsr_stage() == ADSRStage::Attack);
    let ramp = adsr.process(&mut note, 100., 500., 0.5, 1000.);
    assert_approximately_eq!(ramp, 1., 6);
    assert_eq!(adsr.get_speed(), 0.5);
    assert_eq!(adsr.gain, 1.);
  }

  #[test]
  fn can_change_parameters_mid_ramp() {
    let mut note = Note::default();
    let mut adsr = ADSR::new(10., 1000.);

    assert!(*note.get_adsr_stage() == ADSRStage::Idle);
    // attack stage
    note.note_on(60, 1.);
    assert!(*note.get_adsr_stage() == ADSRStage::Attack);
    assert_eq!(adsr.get_speed(), 1.);
    let ramp = adsr.process(&mut note, 1000., 100., 0.5, 1000.);
    assert_approximately_eq!(ramp, 0.1, 6);
    let ramp = adsr.process(&mut note, 500., 100., 0.5, 1000.);
    assert_approximately_eq!(ramp, 0.3, 6);
    let ramp = adsr.process(&mut note, 500., 100., 0.5, 1000.);
    assert_approximately_eq!(ramp, 0.5, 6);
    let ramp = adsr.process(&mut note, 500., 100., 0.5, 1000.);
    assert_approximately_eq!(ramp, 0.7, 6);
    let ramp = adsr.process(&mut note, 500., 100., 0.5, 1000.);
    assert_approximately_eq!(ramp, 0.9, 6);
    let ramp = adsr.process(&mut note, 500., 100., 0.5, 1000.);
    assert_approximately_eq!(ramp, 1., 6);
    // decay stage
    assert!(*note.get_adsr_stage() == ADSRStage::Decay);
    let ramp = adsr.process(&mut note, 1000., 1000., 0.5, 1000.);
    assert_approximately_eq!(ramp, 0.95, 6);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 1000.);
    assert_approximately_eq!(ramp, 0.85, 6);
    // sustain stage
    let ramp = adsr.process(&mut note, 1000., 500., 0.9, 1000.);
    assert!(*note.get_adsr_stage() == ADSRStage::Sustain);
    assert_approximately_eq!(ramp, 0.9, 6);
    // release stage
    note.note_off();
    assert!(*note.get_adsr_stage() == ADSRStage::Release);
    let ramp = adsr.process(&mut note, 1000., 500., 0.9, 900.);
    assert_approximately_eq!(ramp, 0.8, 6);
    let ramp = adsr.process(&mut note, 1000., 500., 0.9, 900.);
    assert_approximately_eq!(ramp, 0.7, 6);
    let ramp = adsr.process(&mut note, 1000., 500., 0.9, 450.);
    assert_approximately_eq!(ramp, 0.5, 6);
  }
}
