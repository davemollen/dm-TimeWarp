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
  release_start_value: Option<f32>,
  retrigger_start_value: Option<f32>,
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
      release_start_value: None,
      retrigger_start_value: None,
    }
  }

  pub fn reset(&mut self) {
    self.x = 0.;
    self.gain = 1.;
    self.speed = 1.;
    self.trigger = false;
    self.release_start_value = None;
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
            self.x = 0.;
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
        let range = match self.release_start_value {
          Some(range) => range,
          None => {
            self.release_start_value = Some(self.adsr_output);
            self.x = 1.;
            self.adsr_output
          }
        };

        let release_step_size = release_time.mstosamps(self.sample_rate).recip();
        let next_x = self.x - release_step_size;
        if next_x <= 0. {
          self.x = 0.;
          self.release_start_value = None;
          note.set_adsr_stage(ADSRStage::Idle);
        } else {
          self.x = next_x;
        }

        self.adsr_output = self.x.cube() * range;
      }
      ADSRStage::Retrigger => {
        let range = match self.retrigger_start_value {
          Some(range) => range,
          None => {
            self.retrigger_start_value = Some(self.adsr_output);
            self.x = 1.;
            self.adsr_output
          }
        };

        let next_x = self.x - self.retrigger_step_size;
        if next_x <= 0. {
          self.x = 0.;
          self.release_start_value = None;
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
    for i in 0..10 {
      let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
      assert_approximately_eq!(adsr.x, (i + 1) as f32 / 10., 6);
      assert_approximately_eq!(ramp, (i + 1) as f32 / 10., 6);
    }

    // decay stage
    assert!(*note.get_adsr_stage() == ADSRStage::Decay);
    for i in 0..5 {
      let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
      assert_approximately_eq!(adsr.x, 1.0 - (i + 1) as f32 / 5., 6);
      if i == 4 {
        assert_approximately_eq!(ramp, 0.5, 6);
      }
    }
    // one extra round of processing to really reach the end
    adsr.process(&mut note, 1000., 500., 0.5, 500.);

    // sustain stage
    assert!(*note.get_adsr_stage() == ADSRStage::Sustain);
    for _ in 0..3 {
      let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
      assert_approximately_eq!(ramp, 0.5, 6);
    }

    // release stage
    note.note_off();
    assert!(*note.get_adsr_stage() == ADSRStage::Release);
    for i in 0..5 {
      let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
      assert_approximately_eq!(adsr.x, 1.0 - (i + 1) as f32 / 5., 6);
      if i == 4 {
        assert_approximately_eq!(ramp, 0., 6);
      }
    }
    // one extra round of processing to really reach the end
    adsr.process(&mut note, 1000., 500., 0.5, 500.);

    // idle stage
    assert!(*note.get_adsr_stage() == ADSRStage::Idle);
  }

  #[test]
  fn should_apply_gain_based_on_velocity() {
    let mut note = Note::default();
    let mut adsr = ADSR::new(10., 1000.);

    assert!(*note.get_adsr_stage() == ADSRStage::Idle);
    note.note_on(60, 0.5);

    assert!(*note.get_adsr_stage() == ADSRStage::Attack);
    let ramp = adsr.process(&mut note, 0., 0., 1., 0.);
    assert_eq!(ramp, 0.70710677);
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
    for i in (0..10).rev() {
      let ramp = adsr.process(&mut note, 1000., 500., 0.5, 500.);
      assert_approximately_eq!(adsr.x, i as f32 / 10., 6);
      if i == 0 {
        assert_approximately_eq!(ramp, 0., 6);
      }
    }

    // attack stage
    assert!(*note.get_adsr_stage() == ADSRStage::Attack);
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
    adsr.process(&mut note, 100., 500., 0.5, 500.);
    let ramp = adsr.process(&mut note, 100., 500., 0.5, 500.);
    assert_approximately_eq!(ramp, 0.60800004, 6);
    assert!(*note.get_adsr_stage() == ADSRStage::Decay);

    // retrigger stage
    note.steal_note(64, 1.);
    assert!(*note.get_adsr_stage() == ADSRStage::Retrigger);
    for i in (0..10).rev() {
      let ramp = adsr.process(&mut note, 100., 500., 0.5, 500.);
      assert_approximately_eq!(adsr.x, i as f32 / 10., 6);
      if i == 0 {
        assert_approximately_eq!(ramp, 0., 6);
      }
    }

    // attack stage
    assert!(*note.get_adsr_stage() == ADSRStage::Attack);
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
    assert!(*note.get_adsr_stage() == ADSRStage::Sustain);

    // retrigger stage
    note.steal_note(64, 1.);
    assert!(*note.get_adsr_stage() == ADSRStage::Retrigger);
    for i in (0..10).rev() {
      let ramp = adsr.process(&mut note, 100., 500., 0.5, 500.);
      assert_approximately_eq!(adsr.x, i as f32 / 10., 6);
      if i == 0 {
        assert_approximately_eq!(ramp, 0., 6);
      }
    }

    // attack stage
    assert!(*note.get_adsr_stage() == ADSRStage::Attack);
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
    adsr.process(&mut note, 100., 100., 0.5, 1000.);
    adsr.process(&mut note, 100., 100., 0.5, 1000.);
    assert!(*note.get_adsr_stage() == ADSRStage::Release);

    // retrigger stage
    note.steal_note(64, 1.);
    assert!(*note.get_adsr_stage() == ADSRStage::Retrigger);
    for i in (0..10).rev() {
      let ramp = adsr.process(&mut note, 100., 500., 0.5, 500.);
      assert_approximately_eq!(adsr.x, i as f32 / 10., 6);
      if i == 0 {
        assert_approximately_eq!(ramp, 0., 6);
      }
    }

    // attack stage
    assert!(*note.get_adsr_stage() == ADSRStage::Attack);
  }

  #[test]
  fn holds_gain_and_speed_until_retrigger_is_finished() {
    let mut note = Note::default();
    let mut adsr = ADSR::new(10., 1000.);

    assert!(*note.get_adsr_stage() == ADSRStage::Idle);
    note.note_on(72, 0.75);

    // attack stage
    assert!(*note.get_adsr_stage() == ADSRStage::Attack);
    assert_eq!(adsr.gain, 1.);
    assert_eq!(adsr.get_speed(), 1.);
    adsr.process(&mut note, 100., 1000., 0.5, 1000.);
    assert_eq!(adsr.gain, 0.8660254);
    assert_eq!(adsr.get_speed(), 2.);

    // decay stage
    assert!(*note.get_adsr_stage() == ADSRStage::Decay);
    adsr.process(&mut note, 100., 500., 0.5, 1000.);
    adsr.process(&mut note, 100., 500., 0.5, 1000.);
    assert!(*note.get_adsr_stage() == ADSRStage::Decay);

    // retrigger stage
    note.steal_note(48, 1.);
    assert!(*note.get_adsr_stage() == ADSRStage::Retrigger);
    for _ in (0..10).rev() {
      assert_eq!(adsr.gain, 0.8660254);
      assert_eq!(adsr.get_speed(), 2.);
      adsr.process(&mut note, 100., 500., 0.5, 1000.);
    }
    assert!(*note.get_adsr_stage() == ADSRStage::Attack);
    adsr.process(&mut note, 100., 500., 0.5, 1000.);
    assert_eq!(adsr.gain, 1.0);
    assert_eq!(adsr.get_speed(), 0.5);
  }

  #[test]
  fn can_change_parameters_mid_ramp() {
    let mut note = Note::default();
    let mut adsr = ADSR::new(10., 1000.);

    assert!(*note.get_adsr_stage() == ADSRStage::Idle);
    note.note_on(60, 1.);

    // attack stage
    assert!(*note.get_adsr_stage() == ADSRStage::Attack);
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
    adsr.process(&mut note, 1000., 1000., 0.5, 1000.);
    assert_approximately_eq!(adsr.x, 0.9, 6);
    adsr.process(&mut note, 1000., 500., 0.5, 1000.);
    assert_approximately_eq!(adsr.x, 0.7, 6);
    adsr.process(&mut note, 1000., 500., 0.5, 1000.);
    assert_approximately_eq!(adsr.x, 0.5, 6);
    adsr.process(&mut note, 1000., 500., 0.5, 1000.);
    assert_approximately_eq!(adsr.x, 0.3, 6);
    adsr.process(&mut note, 1000., 500., 0.5, 1000.);
    assert_approximately_eq!(adsr.x, 0.1, 6);
    adsr.process(&mut note, 1000., 500., 0.5, 1000.);
    assert_approximately_eq!(adsr.x, 0.0, 6);

    // sustain stage
    assert!(*note.get_adsr_stage() == ADSRStage::Sustain);
    let ramp = adsr.process(&mut note, 1000., 500., 0.9, 1000.);
    assert_approximately_eq!(ramp, 0.9, 6);
    let ramp = adsr.process(&mut note, 1000., 500., 0.8, 1000.);
    assert_approximately_eq!(ramp, 0.8, 6);

    // release stage
    note.note_off();
    assert!(*note.get_adsr_stage() == ADSRStage::Release);
    adsr.process(&mut note, 1000., 500., 0.8, 1000.);
    assert_approximately_eq!(adsr.x, 0.9, 6);
    adsr.process(&mut note, 1000., 500., 0.8, 500.);
    assert_approximately_eq!(adsr.x, 0.7, 6);
  }
}
