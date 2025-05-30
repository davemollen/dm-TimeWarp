mod note;
pub use note::{ADSRStage, Note};

pub struct Notes {
  notes: Vec<Note>,
  note_queue: Vec<(u8, f32)>,
  voice_count: usize,
}

impl Notes {
  pub fn new() -> Self {
    Self {
      notes: vec![Note::default(); 8],
      note_queue: Vec::with_capacity(128),
      voice_count: 1,
    }
  }

  pub fn get_notes(&mut self) -> &mut Vec<Note> {
    &mut self.notes
  }

  pub fn note_on(&mut self, note: u8, velocity: f32) {
    match self
      .notes
      .iter_mut()
      .take(self.voice_count)
      .find(|n| *n.get_adsr_stage() == ADSRStage::Idle)
    {
      Some(n) => n.note_on(note, velocity),
      None => {
        let index = self.note_queue.len().checked_sub(self.voice_count);
        match index {
          Some(i) => {
            let note_instance = self
              .notes
              .iter_mut()
              .find(|n| n.get_note() == self.note_queue[i].0);
            match note_instance {
              Some(n) => n.steal_note(note, velocity),
              None => return,
            }
          }
          None => match self.notes.get_mut(self.note_queue.len()) {
            Some(n) => n.steal_note(note, velocity),
            None => return,
          },
        }
      }
    }

    self.note_queue.push((note, velocity));
  }

  pub fn note_off(&mut self, note: u8) {
    self.note_queue.retain(|(n, _)| *n != note);

    match self.notes.iter_mut().find(|n| {
      n.get_note() == note
        && match n.get_adsr_stage() {
          ADSRStage::Idle | ADSRStage::Release => false,
          _ => true,
        }
    }) {
      Some(note_instance) => {
        if self.note_queue.len() < self.voice_count {
          note_instance.note_off();
        } else {
          // reactivate the newest note in queue that's not in notes
          let (note, velocity) = self.note_queue[self.note_queue.len() - self.voice_count];
          note_instance.steal_note(note, velocity);
        }
      }
      None => return,
    };
  }

  pub fn set_voice_count(&mut self, voice_count: usize) {
    if voice_count == self.voice_count {
      return;
    }
    self.notes.iter_mut().for_each(|v| v.reset_note());
    self.voice_count = voice_count;
  }
}

#[cfg(test)]
mod tests {
  use super::{ADSRStage, Note, Notes};

  fn assert_notes_vector(notes: &Vec<Note>, expected_notes: Vec<(u8, ADSRStage)>) {
    notes
      .iter()
      .zip(expected_notes)
      .for_each(|(note, (expected_note, expected_state))| {
        assert_eq!(note.get_note(), expected_note);
        assert!(*note.get_adsr_stage() == expected_state);
      })
  }

  #[test]
  fn default_notes() {
    let mut notes = Notes::new();
    notes.set_voice_count(4);
    assert_notes_vector(
      &notes.notes,
      vec![
        (0, ADSRStage::Idle),
        (0, ADSRStage::Idle),
        (0, ADSRStage::Idle),
        (0, ADSRStage::Idle),
      ],
    );
  }

  #[test]
  fn note_on() {
    let mut notes = Notes::new();
    notes.set_voice_count(3);
    notes.note_on(60, 1.);
    assert_notes_vector(&notes.notes, vec![(60, ADSRStage::Attack)]);
    notes.note_on(64, 1.);
    assert_notes_vector(
      &notes.notes,
      vec![(60, ADSRStage::Attack), (64, ADSRStage::Attack)],
    );
    notes.note_on(67, 1.);
    assert_notes_vector(
      &notes.notes,
      vec![
        (60, ADSRStage::Attack),
        (64, ADSRStage::Attack),
        (67, ADSRStage::Attack),
      ],
    );
  }

  #[test]
  fn note_off() {
    let mut notes = Notes::new();
    notes.set_voice_count(3);
    notes.note_on(60, 1.);
    assert_notes_vector(&notes.notes, vec![(60, ADSRStage::Attack)]);
    assert_notes_vector(
      &notes.notes,
      vec![
        (60, ADSRStage::Attack),
        (0, ADSRStage::Idle),
        (0, ADSRStage::Idle),
      ],
    );
    notes.note_off(60);
    assert_notes_vector(&notes.notes, vec![(60, ADSRStage::Release)]);
    notes.note_on(60, 1.);
    assert_notes_vector(
      &notes.notes,
      vec![(60, ADSRStage::Release), (60, ADSRStage::Attack)],
    );
    notes.note_off(60);
    assert_notes_vector(
      &notes.notes,
      vec![(60, ADSRStage::Release), (60, ADSRStage::Release)],
    );
    notes.note_on(60, 1.);
    assert_notes_vector(
      &notes.notes,
      vec![
        (60, ADSRStage::Release),
        (60, ADSRStage::Release),
        (60, ADSRStage::Attack),
      ],
    );
    notes.note_off(60);
    assert_notes_vector(
      &notes.notes,
      vec![
        (60, ADSRStage::Release),
        (60, ADSRStage::Release),
        (60, ADSRStage::Release),
      ],
    );
    notes.note_on(64, 1.);
    assert_notes_vector(
      &notes.notes,
      vec![
        (64, ADSRStage::Retrigger),
        (60, ADSRStage::Release),
        (60, ADSRStage::Release),
      ],
    );
    notes.note_on(67, 1.);
    assert_notes_vector(
      &notes.notes,
      vec![
        (64, ADSRStage::Retrigger),
        (67, ADSRStage::Retrigger),
        (60, ADSRStage::Release),
      ],
    );
    notes.note_off(64);
    assert_notes_vector(
      &notes.notes,
      vec![
        (64, ADSRStage::Release),
        (67, ADSRStage::Retrigger),
        (60, ADSRStage::Release),
      ],
    );
    notes.note_off(67);
    assert_notes_vector(
      &notes.notes,
      vec![
        (64, ADSRStage::Release),
        (67, ADSRStage::Release),
        (60, ADSRStage::Release),
      ],
    );
  }

  #[test]
  fn steals_in_polyphonic_mode() {
    let mut notes = Notes::new();
    notes.set_voice_count(2);
    notes.note_on(60, 1.);
    assert_notes_vector(&notes.notes, vec![(60, ADSRStage::Attack)]);
    notes.note_on(64, 1.);
    assert_notes_vector(
      &notes.notes,
      vec![(60, ADSRStage::Attack), (64, ADSRStage::Attack)],
    );
    notes.note_on(65, 0.5);
    assert_notes_vector(
      &notes.notes,
      vec![(65, ADSRStage::Retrigger), (64, ADSRStage::Attack)],
    );
    notes.note_on(69, 0.75);
    assert_notes_vector(
      &notes.notes,
      vec![(65, ADSRStage::Retrigger), (69, ADSRStage::Retrigger)],
    );
    notes.note_off(65);
    assert_notes_vector(
      &notes.notes,
      vec![(64, ADSRStage::Retrigger), (69, ADSRStage::Retrigger)],
    );
    notes.note_off(69);
    assert_notes_vector(
      &notes.notes,
      vec![(64, ADSRStage::Retrigger), (60, ADSRStage::Retrigger)],
    );
    notes.note_off(60);
    assert_notes_vector(
      &notes.notes,
      vec![(64, ADSRStage::Retrigger), (60, ADSRStage::Release)],
    );
    notes.note_off(64);
    assert_notes_vector(
      &notes.notes,
      vec![(64, ADSRStage::Release), (60, ADSRStage::Release)],
    );
    notes.note_on(65, 0.5);
    assert_notes_vector(
      &notes.notes,
      vec![(65, ADSRStage::Retrigger), (60, ADSRStage::Release)],
    );
    notes.note_on(69, 0.75);
    assert_notes_vector(
      &notes.notes,
      vec![(65, ADSRStage::Retrigger), (69, ADSRStage::Retrigger)],
    );
  }

  #[test]
  fn steals_in_monophonic_mode() {
    let mut notes = Notes::new();
    notes.set_voice_count(1);
    notes.note_on(60, 1.);
    assert_notes_vector(&notes.notes, vec![(60, ADSRStage::Attack)]);
    notes.note_off(60);
    assert_notes_vector(&notes.notes, vec![(60, ADSRStage::Release)]);
    notes.note_on(60, 1.);
    assert_notes_vector(&notes.notes, vec![(60, ADSRStage::Retrigger)]);
    notes.note_on(59, 0.5);
    assert_notes_vector(&notes.notes, vec![(59, ADSRStage::Retrigger)]);
    notes.note_on(72, 0.75);
    assert_notes_vector(&notes.notes, vec![(72, ADSRStage::Retrigger)]);
    notes.note_off(59);
    assert_notes_vector(&notes.notes, vec![(72, ADSRStage::Retrigger)]);
    notes.note_off(72);
    assert_notes_vector(&notes.notes, vec![(60, ADSRStage::Retrigger)]);
    notes.note_off(60);
    assert_notes_vector(&notes.notes, vec![(60, ADSRStage::Release)]);
    notes.note_on(60, 1.);
    assert_notes_vector(&notes.notes, vec![(60, ADSRStage::Retrigger)]);
  }
}
