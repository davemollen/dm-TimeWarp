mod grains;
mod voice;
use {crate::stereo_delay_line::StereoDelayLine, grains::Grains, voice::Voice};

pub struct Voices {
  voices: Vec<Voice>,
  note_queue: Vec<(u8, f32)>,
  grains: Vec<Grains>,
  voice_count: usize,
}

impl Voices {
  pub fn new(sample_rate: f32, fade_time: f32) -> Self {
    Self {
      voices: Vec::with_capacity(8),
      note_queue: Vec::with_capacity(128),
      grains: vec![Grains::new(sample_rate, fade_time); 8],
      voice_count: 1,
    }
  }

  pub fn process(
    &mut self,
    delay_line: &StereoDelayLine,
    size: f32,
    time: f32,
    density: f32,
    speed: f32,
    stretch: f32,
    scan: f32,
    spray: f32,
    midi_enabled: bool,
  ) -> (f32, f32) {
    if midi_enabled {
      self
        .voices
        .iter()
        .zip(self.grains.iter_mut())
        .fold((0., 0.), |result, (voice, grains)| {
          let grains_out = grains.process(
            delay_line,
            size,
            time,
            density,
            speed * voice.get_speed(),
            stretch,
            scan,
            spray,
          );
          (
            result.0 + grains_out.0 * voice.get_gain(),
            result.1 + grains_out.1 * voice.get_gain(),
          )
        })
    } else {
      self.grains[0].process(delay_line, size, time, density, speed, stretch, scan, spray)
    }
  }

  pub fn note_on(&mut self, note: u8, velocity: f32) {
    if self.voices.len() < self.voice_count {
      self.voices.push(Voice::note_on(note, velocity));
    } else {
      let index = self.note_queue.len().checked_sub(self.voice_count);
      match index {
        Some(i) => {
          let voice = self
            .voices
            .iter_mut()
            .find(|v| v.get_note() == self.note_queue[i].0);
          match voice {
            Some(v) => v.steal_note(note, velocity),
            None => return,
          }
        }
        None => self.voices[self.note_queue.len()].steal_note(note, velocity),
      }
    }
    self.note_queue.push((note, velocity));
  }

  pub fn note_off(&mut self, note: u8) {
    self.note_queue.retain(|(n, _)| *n != note);

    match self
      .voices
      .iter_mut()
      .find(|v| v.get_note() == note && v.get_gain() > 0.)
    {
      Some(voice) => {
        if self.note_queue.len() < self.voice_count {
          voice.note_off();
        } else {
          // reactivate the newest note in queue that's not in voices
          let (note, velocity) = self.note_queue[self.note_queue.len() - self.voice_count];
          voice.steal_note(note, velocity);
        }
      }
      None => return,
    };
  }

  pub fn set_voice_count(&mut self, voice_count: usize) {
    if self.voice_count > voice_count {
      self
        .voices
        .iter_mut()
        .skip(voice_count)
        .for_each(|v| v.note_off());
    }
    self.voice_count = voice_count;
  }
}

#[cfg(test)]
mod tests {
  use super::{Voice, Voices};

  fn assert_voices_vector(voices: &Vec<Voice>, expected_notes: Vec<(u8, f32)>) {
    voices
      .iter()
      .zip(expected_notes)
      .for_each(|(voice, (expected_note, expected_gain))| {
        assert_eq!(voice.get_note(), expected_note);
        assert_eq!(voice.get_gain(), expected_gain);
      })
  }

  #[test]
  fn adds_notes() {
    let mut voices = Voices::new(44100., 10.);
    voices.set_voice_count(3);
    voices.note_on(60, 1.);
    assert_voices_vector(&voices.voices, vec![(60, 1.)]);
    voices.note_on(64, 1.);
    assert_voices_vector(&voices.voices, vec![(60, 1.), (64, 1.)]);
    voices.note_on(67, 1.);
    assert_voices_vector(&voices.voices, vec![(60, 1.), (64, 1.), (67, 1.)]);
  }

  #[test]
  fn removes_notes() {
    let mut voices = Voices::new(44100., 10.);
    voices.set_voice_count(3);
    voices.note_on(60, 1.);
    assert_voices_vector(&voices.voices, vec![(60, 1.)]);
    voices.note_off(60);
    assert_voices_vector(&voices.voices, vec![(60, 0.)]);
    voices.note_on(60, 1.);
    assert_voices_vector(&voices.voices, vec![(60, 0.), (60, 1.)]);
    voices.note_off(60);
    assert_voices_vector(&voices.voices, vec![(60, 0.), (60, 0.)]);
    voices.note_on(60, 1.);
    assert_voices_vector(&voices.voices, vec![(60, 0.), (60, 0.), (60, 1.)]);
    voices.note_off(60);
    assert_voices_vector(&voices.voices, vec![(60, 0.), (60, 0.), (60, 0.)]);

    voices.note_on(60, 1.);
    voices.note_on(64, 1.);
    voices.note_on(67, 1.);
    assert_voices_vector(&voices.voices, vec![(60, 1.), (64, 1.), (67, 1.)]);
    voices.note_off(60);
    assert_voices_vector(&voices.voices, vec![(60, 0.), (64, 1.), (67, 1.)]);
    voices.note_off(64);
    assert_voices_vector(&voices.voices, vec![(60, 0.), (64, 0.), (67, 1.)]);
    voices.note_off(67);
    assert_voices_vector(&voices.voices, vec![(60, 0.), (64, 0.), (67, 0.)]);
  }

  #[test]
  fn polyphonic_steals() {
    let mut voices = Voices::new(44100., 10.);
    voices.set_voice_count(2);
    voices.note_on(60, 1.);
    assert_eq!(voices.note_queue.len(), 1);
    assert_voices_vector(&voices.voices, vec![(60, 1.)]);
    voices.note_on(64, 1.);
    assert_eq!(voices.note_queue.len(), 2);
    assert_voices_vector(&voices.voices, vec![(60, 1.), (64, 1.)]);
    voices.note_on(65, 0.5);
    assert_eq!(voices.note_queue.len(), 3);
    assert_voices_vector(&voices.voices, vec![(65, 0.5), (64, 1.)]);
    voices.note_on(69, 0.75);
    assert_eq!(voices.note_queue.len(), 4);
    assert_voices_vector(&voices.voices, vec![(65, 0.5), (69, 0.75)]);
    voices.note_off(65);
    assert_eq!(voices.note_queue.len(), 3);
    assert_voices_vector(&voices.voices, vec![(64, 1.), (69, 0.75)]);
    voices.note_off(69);
    assert_eq!(voices.note_queue.len(), 2);
    assert_voices_vector(&voices.voices, vec![(64, 1.), (60, 1.)]);
    voices.note_off(60);
    assert_eq!(voices.note_queue.len(), 1);
    assert_voices_vector(&voices.voices, vec![(64, 1.), (60, 0.)]);
    voices.note_off(64);
    assert_eq!(voices.note_queue.len(), 0);
    assert_voices_vector(&voices.voices, vec![(64, 0.), (60, 0.)]);
    voices.note_on(65, 0.5);
    assert_eq!(voices.note_queue.len(), 1);
    assert_voices_vector(&voices.voices, vec![(65, 0.5), (60, 0.)]);
    voices.note_on(69, 0.75);
    assert_eq!(voices.note_queue.len(), 2);
    assert_voices_vector(&voices.voices, vec![(65, 0.5), (69, 0.75)]);
  }

  #[test]
  fn monophonic_steals() {
    let mut voices = Voices::new(44100., 10.);
    voices.set_voice_count(1);
    voices.note_on(60, 1.);
    assert_eq!(voices.note_queue.len(), 1);
    assert_voices_vector(&voices.voices, vec![(60, 1.)]);
    voices.note_off(60);
    assert_eq!(voices.note_queue.len(), 0);
    assert_voices_vector(&voices.voices, vec![(60, 0.)]);
    voices.note_on(60, 1.);
    assert_eq!(voices.note_queue.len(), 1);
    assert_voices_vector(&voices.voices, vec![(60, 1.)]);
    voices.note_on(59, 0.5);
    assert_voices_vector(&voices.voices, vec![(59, 0.5)]);
    voices.note_on(72, 0.75);
    assert_voices_vector(&voices.voices, vec![(72, 0.75)]);
    voices.note_off(59);
    assert_voices_vector(&voices.voices, vec![(72, 0.75)]);
    voices.note_off(72);
    assert_voices_vector(&voices.voices, vec![(60, 1.)]);
    voices.note_off(60);
    assert_voices_vector(&voices.voices, vec![(60, 0.)]);
    voices.note_on(60, 1.);
    assert_voices_vector(&voices.voices, vec![(60, 1.)]);
  }
}
