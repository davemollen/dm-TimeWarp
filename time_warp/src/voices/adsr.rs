use crate::notes::{ADSRStage, Note};

#[derive(Clone)]
pub struct ADSR {
  x: f32,
  gain: f32,
  speed: f32,
  trigger: bool,
  t: f32,
  retrigger_time: f32,
}

impl ADSR {
  pub fn new(sample_rate: f32, retrigger_time: f32) -> Self {
    Self {
      x: 0.,
      gain: 1.,
      speed: 1.,
      trigger: false,
      t: sample_rate.recip(),
      retrigger_time,
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
        if (1. - self.x) <= f32::EPSILON {
          self.x = 1.;
          note.set_adsr_stage(ADSRStage::Decay);
        } else {
          self.apply_curve(1., attack_time);
        }
      }
      ADSRStage::Decay => {
        if (self.x - sustain) <= f32::EPSILON {
          self.x = sustain;
          note.set_adsr_stage(ADSRStage::Sustain);
        } else {
          self.apply_curve(sustain, decay_time);
        }
      }
      ADSRStage::Sustain => {
        self.x = sustain;
      }
      ADSRStage::Release => {
        if self.x <= f32::EPSILON {
          self.x = 0.;
          note.set_adsr_stage(ADSRStage::Idle);
        } else {
          self.apply_curve(0., release_time);
        }
      }
      ADSRStage::Retrigger => {
        if self.x <= f32::EPSILON {
          self.x = 0.;
          note.set_adsr_stage(ADSRStage::Attack);
        } else {
          self.apply_curve(0., self.retrigger_time);
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

  fn apply_curve(&mut self, target: f32, time: f32) {
    let b1 = (-self.t / (time / 6910.)).exp();
    let a0 = 1. - b1;

    self.x = target * a0 + self.x * b1;
  }
}
