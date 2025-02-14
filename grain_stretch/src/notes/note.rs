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
pub struct Note {
  note: u8,
  speed: f32,
  gain: f32,
  adsr_stage: ADSRStage,
}

impl Note {
  pub fn default() -> Self {
    Self {
      note: 0,
      speed: 0.,
      gain: 0.,
      adsr_stage: ADSRStage::Idle,
    }
  }

  pub fn note_on(&mut self, note: u8, velocity: f32) {
    self.note = note;
    self.speed = Self::calculate_speed(note);
    self.gain = velocity;
    self.adsr_stage = ADSRStage::Attack;
  }

  pub fn note_off(&mut self) {
    self.adsr_stage = ADSRStage::Release;
  }

  pub fn steal_note(&mut self, note: u8, velocity: f32) {
    self.note = note;
    self.speed = Self::calculate_speed(note);
    self.gain = velocity;
    self.adsr_stage = match self.adsr_stage {
      ADSRStage::Idle => ADSRStage::Attack,
      _ => ADSRStage::Retrigger,
    };
  }

  pub fn reset_note(&mut self) {
    self.note = 0;
    self.speed = 0.;
    self.gain = 0.;
    self.adsr_stage = ADSRStage::Idle;
  }

  pub fn set_adsr_stage(&mut self, adsr_stage: ADSRStage) {
    self.adsr_stage = adsr_stage;
  }

  pub fn get_note(&self) -> u8 {
    self.note
  }

  pub fn get_speed(&self) -> f32 {
    self.speed
  }

  pub fn get_gain(&self) -> f32 {
    self.gain
  }

  pub fn get_adsr_stage(&self) -> &ADSRStage {
    &self.adsr_stage
  }

  fn calculate_speed(note: u8) -> f32 {
    2_f32.powf((note as f32 - 60.).clamp(-48., 48.) / 12.)
  }
}
