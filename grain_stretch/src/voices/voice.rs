pub struct Voice {
  note: u8,
  speed: f32,
  gain: f32,
}

impl Voice {
  pub fn note_on(note: u8, velocity: f32) -> Self {
    let speed = Self::calculate_speed(note);
    let gain = velocity;

    Self { note, speed, gain }
  }

  pub fn note_off(&mut self) {
    self.gain = 0.;
  }

  pub fn steal_note(&mut self, note: u8, velocity: f32) {
    let speed = Self::calculate_speed(note);
    let gain = velocity;

    self.note = note;
    self.speed = speed;
    self.gain = gain;
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

  fn calculate_speed(note: u8) -> f32 {
    2_f32.powf((note as f32 - 60.).clamp(-48., 48.) / 12.)
  }
}
