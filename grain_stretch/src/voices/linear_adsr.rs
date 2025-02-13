use crate::{
  notes::{ADSRStage, Note},
  shared::float_ext::FloatExt,
};

#[derive(Clone)]
pub struct ADSR {
  x: f32,
  sample_rate: f32,
  retrigger_step_size: f32,
  gain: f32,
  speed: f32,
  trigger: bool,
}

impl ADSR {
  pub fn new(sample_rate: f32, retrigger_time: f32) -> Self {
    let retrigger_step_size = retrigger_time.mstosamps(sample_rate).recip();
    Self {
      x: 0.,
      sample_rate,
      retrigger_step_size,
      gain: 1.,
      speed: 1.,
      trigger: false,
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
    match note.get_adsr_stage() {
      ADSRStage::Idle => {
        self.x = 0.;
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
      }
      ADSRStage::Decay => {
        let decay_step_size = decay_time.mstosamps(self.sample_rate).recip() * (1. - sustain);
        let next_x = self.x - decay_step_size;
        if next_x <= sustain {
          self.x = sustain;
          note.set_adsr_stage(ADSRStage::Sustain);
        } else {
          self.x = next_x;
        }
      }
      ADSRStage::Sustain => {
        self.x = sustain;
      }
      ADSRStage::Release => {
        let release_step_size = release_time.mstosamps(self.sample_rate).recip() * sustain;
        let next_x = self.x - release_step_size;
        if next_x <= 0. {
          self.x = 0.;
          note.set_adsr_stage(ADSRStage::Idle);
        } else {
          self.x = next_x;
        }
      }
      ADSRStage::Retrigger => {
        let next_x = self.x - self.retrigger_step_size;
        if next_x <= 0. {
          self.x = 0.;
          note.set_adsr_stage(ADSRStage::Attack);
        } else {
          self.x = next_x;
        }
      }
    };

    self.x * self.gain
  }

  pub fn get_speed(&self) -> f32 {
    self.speed
  }

  pub fn get_trigger(&self) -> bool {
    self.trigger
  }
}

#[cfg(test)]
mod tests {
  use super::ADSR;
  use crate::notes::{ADSRStage, Note};

  fn assert_approx_eq(left: f32, right: f32) {
    let left = (left * 100000.).round() / 100000.;
    let right = (right * 100000.).round() / 100000.;
    assert_eq!(left, right);
  }

  #[test]
  fn regular_adsr() {
    let mut note = Note::default();
    let mut adsr = ADSR::new(10., 1000.);

    assert!(*note.get_adsr_stage() == ADSRStage::Idle);
    note.note_on(60, 1.);
    // attack stage
    assert!(*note.get_adsr_stage() == ADSRStage::Attack);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approx_eq(ramp, 0.1);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approx_eq(ramp, 0.2);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approx_eq(ramp, 0.3);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approx_eq(ramp, 0.4);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approx_eq(ramp, 0.5);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approx_eq(ramp, 0.6);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approx_eq(ramp, 0.7);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approx_eq(ramp, 0.8);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approx_eq(ramp, 0.9);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approx_eq(ramp, 1.0);
    // decay stage
    assert!(*note.get_adsr_stage() == ADSRStage::Decay);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approx_eq(ramp, 0.9);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approx_eq(ramp, 0.8);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approx_eq(ramp, 0.7);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approx_eq(ramp, 0.6);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approx_eq(ramp, 0.5);
    // sustain stage
    assert!(*note.get_adsr_stage() == ADSRStage::Sustain);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approx_eq(ramp, 0.5);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approx_eq(ramp, 0.5);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approx_eq(ramp, 0.5);
    // release stage
    note.note_off();
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approx_eq(ramp, 0.4);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approx_eq(ramp, 0.3);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approx_eq(ramp, 0.2);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approx_eq(ramp, 0.1);
    assert!(*note.get_adsr_stage() == ADSRStage::Release);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approx_eq(ramp, 0.);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approx_eq(ramp, 0.);
    assert!(*note.get_adsr_stage() == ADSRStage::Idle);
  }

  #[test]
  fn should_apply_gain_based_on_velocity() {
    let mut note = Note::default();
    let mut adsr = ADSR::new(10., 1000.);
    assert!(*note.get_adsr_stage() == ADSRStage::Idle);
    note.note_on(60, 0.5);
    let ramp = adsr.process(&mut note, 0., 0., 1., 0.);
    assert_eq!(ramp, 0.5);
    let ramp = adsr.process(&mut note, 0., 0., 0.5, 0.);
    assert_eq!(ramp, 0.25);
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
    assert_approx_eq(ramp, 0.1);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approx_eq(ramp, 0.2);
    // retrigger stage
    note.steal_note(64, 1.);
    assert!(*note.get_adsr_stage() == ADSRStage::Retrigger);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approx_eq(ramp, 0.1);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approx_eq(ramp, 0.);
    // attack stage
    assert!(*note.get_adsr_stage() == ADSRStage::Attack);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approx_eq(ramp, 0.1);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
    assert_approx_eq(ramp, 0.2);
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
    assert_approx_eq(ramp, 1.);
    // decay stage
    assert!(*note.get_adsr_stage() == ADSRStage::Decay);
    let ramp = adsr.process(&mut note, 100., 500., 0.5, 500.);
    assert_approx_eq(ramp, 0.9);
    let ramp = adsr.process(&mut note, 100., 500., 0.5, 500.);
    assert_approx_eq(ramp, 0.8);
    // retrigger stage
    note.steal_note(64, 1.);
    assert!(*note.get_adsr_stage() == ADSRStage::Retrigger);
    let ramp = adsr.process(&mut note, 100., 500., 0.5, 500.);
    assert_approx_eq(ramp, 0.7);
    let ramp = adsr.process(&mut note, 100., 500., 0.5, 500.);
    assert_approx_eq(ramp, 0.6);
    let ramp = adsr.process(&mut note, 100., 500., 0.5, 500.);
    assert_approx_eq(ramp, 0.5);
    let ramp = adsr.process(&mut note, 100., 500., 0.5, 500.);
    assert_approx_eq(ramp, 0.4);
    let ramp = adsr.process(&mut note, 100., 500., 0.5, 500.);
    assert_approx_eq(ramp, 0.3);
    let ramp = adsr.process(&mut note, 100., 500., 0.5, 500.);
    assert_approx_eq(ramp, 0.2);
    let ramp = adsr.process(&mut note, 100., 500., 0.5, 500.);
    assert_approx_eq(ramp, 0.1);
    assert!(*note.get_adsr_stage() == ADSRStage::Retrigger);
    let ramp = adsr.process(&mut note, 100., 500., 0.5, 500.);
    // attack stage
    assert!(*note.get_adsr_stage() == ADSRStage::Attack);
    assert_approx_eq(ramp, 0.);
    let ramp = adsr.process(&mut note, 100., 500., 0.5, 500.);
    assert_approx_eq(ramp, 1.);
    // decay stage
    assert!(*note.get_adsr_stage() == ADSRStage::Decay);
    let ramp = adsr.process(&mut note, 100., 500., 0.5, 500.);
    assert_approx_eq(ramp, 0.9);
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
    assert_approx_eq(ramp, 1.);
    // decay stage
    assert!(*note.get_adsr_stage() == ADSRStage::Decay);
    let ramp = adsr.process(&mut note, 100., 100., 0.5, 500.);
    assert_approx_eq(ramp, 0.5);
    // sustain stage
    assert!(*note.get_adsr_stage() == ADSRStage::Sustain);
    let ramp = adsr.process(&mut note, 100., 100., 0.5, 500.);
    assert_approx_eq(ramp, 0.5);
    let ramp = adsr.process(&mut note, 100., 100., 0.5, 500.);
    assert_approx_eq(ramp, 0.5);
    // retrigger stage
    note.steal_note(64, 1.);
    assert!(*note.get_adsr_stage() == ADSRStage::Retrigger);
    let ramp = adsr.process(&mut note, 100., 100., 0.5, 500.);
    assert_approx_eq(ramp, 0.4);
    let ramp = adsr.process(&mut note, 100., 100., 0.5, 500.);
    assert_approx_eq(ramp, 0.3);
    let ramp = adsr.process(&mut note, 100., 100., 0.5, 500.);
    assert_approx_eq(ramp, 0.2);
    let ramp = adsr.process(&mut note, 100., 100., 0.5, 500.);
    assert_approx_eq(ramp, 0.1);
    let ramp = adsr.process(&mut note, 100., 100., 0.5, 500.);
    assert_approx_eq(ramp, 0.);
    let ramp = adsr.process(&mut note, 100., 100., 0.5, 500.);
    assert_approx_eq(ramp, 0.);
    // attack stage
    assert!(*note.get_adsr_stage() == ADSRStage::Attack);
    let ramp = adsr.process(&mut note, 100., 100., 0.5, 500.);
    assert_approx_eq(ramp, 1.);
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
    assert_approx_eq(ramp, 1.);
    // decay stage
    assert!(*note.get_adsr_stage() == ADSRStage::Decay);
    let ramp = adsr.process(&mut note, 100., 100., 0.5, 1000.);
    assert_approx_eq(ramp, 0.5);
    // sustain stage
    assert!(*note.get_adsr_stage() == ADSRStage::Sustain);
    let ramp = adsr.process(&mut note, 100., 100., 0.5, 1000.);
    assert_approx_eq(ramp, 0.5);
    // release stage
    note.note_off();
    assert!(*note.get_adsr_stage() == ADSRStage::Release);
    let ramp = adsr.process(&mut note, 100., 100., 0.5, 1000.);
    assert_approx_eq(ramp, 0.45);
    assert!(*note.get_adsr_stage() == ADSRStage::Release);
    let ramp = adsr.process(&mut note, 100., 100., 0.5, 1000.);
    assert_approx_eq(ramp, 0.4);
    // retrigger stage
    note.steal_note(64, 1.);
    assert!(*note.get_adsr_stage() == ADSRStage::Retrigger);
    let ramp = adsr.process(&mut note, 100., 100., 0.5, 1000.);
    assert_approx_eq(ramp, 0.3);
    let ramp = adsr.process(&mut note, 100., 100., 0.5, 1000.);
    assert_approx_eq(ramp, 0.2);
    let ramp = adsr.process(&mut note, 100., 100., 0.5, 1000.);
    assert_approx_eq(ramp, 0.1);
    let ramp = adsr.process(&mut note, 100., 100., 0.5, 1000.);
    assert_approx_eq(ramp, 0.);

    // attack stage
    assert!(*note.get_adsr_stage() == ADSRStage::Attack);
    let ramp = adsr.process(&mut note, 100., 100., 0.5, 1000.);
    assert_approx_eq(ramp, 1.);
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
    assert_approx_eq(ramp, 0.75);
    // decay stage
    assert!(*note.get_adsr_stage() == ADSRStage::Decay);
    let ramp = adsr.process(&mut note, 100., 500., 0.5, 1000.);
    assert_approx_eq(ramp, 0.675);
    let ramp = adsr.process(&mut note, 100., 500., 0.5, 1000.);
    assert_approx_eq(ramp, 0.6);
    // retrigger stage
    note.steal_note(48, 1.);
    assert!(*note.get_adsr_stage() == ADSRStage::Retrigger);
    assert_eq!(adsr.get_speed(), 2.);
    assert_eq!(adsr.gain, 0.75);
    let ramp = adsr.process(&mut note, 100., 500., 0.5, 1000.);
    assert_eq!(adsr.get_speed(), 2.);
    assert_eq!(adsr.gain, 0.75);
    assert_approx_eq(ramp, 0.525);
    let ramp = adsr.process(&mut note, 100., 500., 0.5, 1000.);
    assert_approx_eq(ramp, 0.45);
    let ramp = adsr.process(&mut note, 100., 500., 0.5, 1000.);
    assert_approx_eq(ramp, 0.375);
    let ramp = adsr.process(&mut note, 100., 500., 0.5, 1000.);
    assert_approx_eq(ramp, 0.3);
    let ramp = adsr.process(&mut note, 100., 500., 0.5, 1000.);
    assert_approx_eq(ramp, 0.225);
    let ramp = adsr.process(&mut note, 100., 500., 0.5, 1000.);
    assert_approx_eq(ramp, 0.15);
    let ramp = adsr.process(&mut note, 100., 500., 0.5, 1000.);
    assert_approx_eq(ramp, 0.075);
    assert_eq!(adsr.get_speed(), 2.);
    assert_eq!(adsr.gain, 0.75);
    let ramp = adsr.process(&mut note, 100., 500., 0.5, 1000.);
    assert_approx_eq(ramp, 0.);
    // attack stage
    assert!(*note.get_adsr_stage() == ADSRStage::Attack);
    let ramp = adsr.process(&mut note, 100., 500., 0.5, 1000.);
    assert_approx_eq(ramp, 1.);
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
    assert_approx_eq(ramp, 0.1);
    let ramp = adsr.process(&mut note, 500., 100., 0.5, 1000.);
    assert_approx_eq(ramp, 0.3);
    let ramp = adsr.process(&mut note, 500., 100., 0.5, 1000.);
    assert_approx_eq(ramp, 0.5);
    let ramp = adsr.process(&mut note, 500., 100., 0.5, 1000.);
    assert_approx_eq(ramp, 0.7);
    let ramp = adsr.process(&mut note, 500., 100., 0.5, 1000.);
    assert_approx_eq(ramp, 0.9);
    let ramp = adsr.process(&mut note, 500., 100., 0.5, 1000.);
    assert_approx_eq(ramp, 1.);
    // decay stage
    assert!(*note.get_adsr_stage() == ADSRStage::Decay);
    let ramp = adsr.process(&mut note, 1000., 1000., 0.5, 1000.);
    assert_approx_eq(ramp, 0.95);
    let ramp = adsr.process(&mut note, 1000., 500., 0.5, 1000.);
    assert_approx_eq(ramp, 0.85);
    // sustain stage
    let ramp = adsr.process(&mut note, 1000., 500., 0.9, 1000.);
    assert!(*note.get_adsr_stage() == ADSRStage::Sustain);
    assert_approx_eq(ramp, 0.9);
    // release stage
    note.note_off();
    assert!(*note.get_adsr_stage() == ADSRStage::Release);
    let ramp = adsr.process(&mut note, 1000., 500., 0.9, 900.);
    assert_approx_eq(ramp, 0.8);
    let ramp = adsr.process(&mut note, 1000., 500., 0.9, 900.);
    assert_approx_eq(ramp, 0.7);
    let ramp = adsr.process(&mut note, 1000., 500., 0.9, 450.);
    assert_approx_eq(ramp, 0.5);
  }
}
