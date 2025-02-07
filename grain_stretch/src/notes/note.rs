#[derive(PartialEq)]
pub enum NoteState {
  Idle,
  On,
  Off,
  Stolen,
}

pub struct Note {
  note: u8,
  speed: f32,
  gain: f32,
  state: NoteState,
}

impl Note {
  pub fn note_on(note: u8, velocity: f32) -> Self {
    let speed = Self::calculate_speed(note);
    let gain = velocity;

    Self {
      note,
      speed,
      gain,
      state: NoteState::On,
    }
  }

  pub fn note_off(&mut self) {
    self.state = NoteState::Off;
  }

  pub fn steal_note(&mut self, note: u8, velocity: f32) {
    let speed = Self::calculate_speed(note);
    let gain = velocity;

    self.note = note;
    self.speed = speed;
    self.gain = gain;
    self.state = match self.state {
      NoteState::Idle => NoteState::On,
      _ => NoteState::Stolen,
    };
  }

  pub fn set_state(&mut self, state: NoteState) {
    self.state = state;
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

  pub fn get_state(&self) -> &NoteState {
    &self.state
  }

  fn calculate_speed(note: u8) -> f32 {
    2_f32.powf((note as f32 - 60.).clamp(-48., 48.) / 12.)
  }
}
