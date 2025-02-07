mod note;
pub use note::{Note, NoteState};

pub struct Notes {
  notes: Vec<Note>,
  note_queue: Vec<(u8, f32)>,
  voice_count: usize,
}

impl Notes {
  pub fn new() -> Self {
    Self {
      notes: Vec::with_capacity(8),
      note_queue: Vec::with_capacity(128),
      voice_count: 1,
    }
  }

  pub fn get_notes(&mut self) -> &mut Vec<Note> {
    &mut self.notes
  }

  pub fn note_on(&mut self, note: u8, velocity: f32) {
    if self.notes.len() < self.voice_count {
      self.notes.push(Note::note_on(note, velocity));
    } else {
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
        None => self.notes[self.note_queue.len()].steal_note(note, velocity),
      }
    }
    self.note_queue.push((note, velocity));
  }

  pub fn note_off(&mut self, note: u8) {
    self.note_queue.retain(|(n, _)| *n != note);

    match self
      .notes
      .iter_mut()
      .find(|n| n.get_note() == note && n.get_gain() > 0.)
    {
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
    if self.voice_count > voice_count {
      self
        .notes
        .iter_mut()
        .skip(voice_count)
        .for_each(|v| v.note_off());
    }
    self.voice_count = voice_count;
  }

  pub fn remove_note(&mut self, note: &Note) {
    // remove note from notes vector
  }
}

#[cfg(test)]
mod tests {
  use super::{Note, Notes};

  fn assert_notes_vector(notes: &Vec<Note>, expected_notes: Vec<(u8, f32)>) {
    notes
      .iter()
      .zip(expected_notes)
      .for_each(|(note, (expected_note, expected_gain))| {
        assert_eq!(note.get_note(), expected_note);
        assert_eq!(note.get_gain(), expected_gain);
      })
  }

  #[test]
  fn adds_notes() {
    let mut notes = Notes::new();
    notes.set_voice_count(3);
    notes.note_on(60, 1.);
    assert_notes_vector(&notes.notes, vec![(60, 1.)]);
    notes.note_on(64, 1.);
    assert_notes_vector(&notes.notes, vec![(60, 1.), (64, 1.)]);
    notes.note_on(67, 1.);
    assert_notes_vector(&notes.notes, vec![(60, 1.), (64, 1.), (67, 1.)]);
  }

  #[test]
  fn removes_notes() {
    let mut notes = Notes::new();
    notes.set_voice_count(3);
    notes.note_on(60, 1.);
    assert_notes_vector(&notes.notes, vec![(60, 1.)]);
    notes.note_off(60);
    assert_notes_vector(&notes.notes, vec![(60, 0.)]);
    notes.note_on(60, 1.);
    assert_notes_vector(&notes.notes, vec![(60, 0.), (60, 1.)]);
    notes.note_off(60);
    assert_notes_vector(&notes.notes, vec![(60, 0.), (60, 0.)]);
    notes.note_on(60, 1.);
    assert_notes_vector(&notes.notes, vec![(60, 0.), (60, 0.), (60, 1.)]);
    notes.note_off(60);
    assert_notes_vector(&notes.notes, vec![(60, 0.), (60, 0.), (60, 0.)]);

    notes.note_on(60, 1.);
    notes.note_on(64, 1.);
    notes.note_on(67, 1.);
    assert_notes_vector(&notes.notes, vec![(60, 1.), (64, 1.), (67, 1.)]);
    notes.note_off(60);
    assert_notes_vector(&notes.notes, vec![(60, 0.), (64, 1.), (67, 1.)]);
    notes.note_off(64);
    assert_notes_vector(&notes.notes, vec![(60, 0.), (64, 0.), (67, 1.)]);
    notes.note_off(67);
    assert_notes_vector(&notes.notes, vec![(60, 0.), (64, 0.), (67, 0.)]);
  }

  #[test]
  fn steals_in_polyphonic_mode() {
    let mut notes = Notes::new();
    notes.set_voice_count(2);
    notes.note_on(60, 1.);
    assert_eq!(notes.note_queue.len(), 1);
    assert_notes_vector(&notes.notes, vec![(60, 1.)]);
    notes.note_on(64, 1.);
    assert_eq!(notes.note_queue.len(), 2);
    assert_notes_vector(&notes.notes, vec![(60, 1.), (64, 1.)]);
    notes.note_on(65, 0.5);
    assert_eq!(notes.note_queue.len(), 3);
    assert_notes_vector(&notes.notes, vec![(65, 0.5), (64, 1.)]);
    notes.note_on(69, 0.75);
    assert_eq!(notes.note_queue.len(), 4);
    assert_notes_vector(&notes.notes, vec![(65, 0.5), (69, 0.75)]);
    notes.note_off(65);
    assert_eq!(notes.note_queue.len(), 3);
    assert_notes_vector(&notes.notes, vec![(64, 1.), (69, 0.75)]);
    notes.note_off(69);
    assert_eq!(notes.note_queue.len(), 2);
    assert_notes_vector(&notes.notes, vec![(64, 1.), (60, 1.)]);
    notes.note_off(60);
    assert_eq!(notes.note_queue.len(), 1);
    assert_notes_vector(&notes.notes, vec![(64, 1.), (60, 0.)]);
    notes.note_off(64);
    assert_eq!(notes.note_queue.len(), 0);
    assert_notes_vector(&notes.notes, vec![(64, 0.), (60, 0.)]);
    notes.note_on(65, 0.5);
    assert_eq!(notes.note_queue.len(), 1);
    assert_notes_vector(&notes.notes, vec![(65, 0.5), (60, 0.)]);
    notes.note_on(69, 0.75);
    assert_eq!(notes.note_queue.len(), 2);
    assert_notes_vector(&notes.notes, vec![(65, 0.5), (69, 0.75)]);
  }

  #[test]
  fn steals_in_monophonic_mode() {
    let mut notes = Notes::new();
    notes.set_voice_count(1);
    notes.note_on(60, 1.);
    assert_eq!(notes.note_queue.len(), 1);
    assert_notes_vector(&notes.notes, vec![(60, 1.)]);
    notes.note_off(60);
    assert_eq!(notes.note_queue.len(), 0);
    assert_notes_vector(&notes.notes, vec![(60, 0.)]);
    notes.note_on(60, 1.);
    assert_eq!(notes.note_queue.len(), 1);
    assert_notes_vector(&notes.notes, vec![(60, 1.)]);
    notes.note_on(59, 0.5);
    assert_notes_vector(&notes.notes, vec![(59, 0.5)]);
    notes.note_on(72, 0.75);
    assert_notes_vector(&notes.notes, vec![(72, 0.75)]);
    notes.note_off(59);
    assert_notes_vector(&notes.notes, vec![(72, 0.75)]);
    notes.note_off(72);
    assert_notes_vector(&notes.notes, vec![(60, 1.)]);
    notes.note_off(60);
    assert_notes_vector(&notes.notes, vec![(60, 0.)]);
    notes.note_on(60, 1.);
    assert_notes_vector(&notes.notes, vec![(60, 1.)]);
  }
}
