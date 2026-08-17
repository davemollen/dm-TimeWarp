use {
  crate::{worker::WorkRequest, AudioFeatures, DmTimeWarp, Ports},
  lv2::prelude::*,
  wmidi::MidiMessage,
};

impl DmTimeWarp {
  pub fn handle_events(&mut self, ports: &mut Ports, features: &mut AudioFeatures) {
    let control_sequence = match ports
      .control
      .read(self.urids.atom.sequence, self.urids.unit.beat)
    {
      Some(sequence_iter) => sequence_iter,
      None => return,
    };

    for (time_stamp, atom) in control_sequence {
      self.time_stamp = time_stamp.as_frames().unwrap_or(0);
      self.read_patch_get_events(atom, ports);
      self.read_patch_set_events(atom, features);
      if self.params.midi_enabled {
        self.read_midi_events(atom);
      }
    }
  }

  pub fn write_set_file(&mut self, ports: &mut Ports) {
    let mut notify_sequence = match ports.notify.init(
      self.urids.atom.sequence,
      TimeStampURID::Frames(self.urids.unit.frame),
    ) {
      Some(sequence_iter) => sequence_iter,
      None => return,
    };

    // Every writer returns None when the notify port runs out of space, which a
    // long file path can do. Bail out instead of panicking on the audio thread.
    let Some(mut object_writer) = notify_sequence.init(
      TimeStamp::Frames(self.time_stamp),
      self.urids.atom.object,
      ObjectHeader {
        id: None,
        otype: self.urids.patch.set_class.into_general(),
      },
    ) else {
      return;
    };
    if object_writer
      .init(
        self.urids.patch.property,
        self.urids.atom.urid,
        self.urids.sample.into_general(),
      )
      .is_none()
    {
      return;
    }
    let Some(mut path_value_writer) =
      object_writer.init(self.urids.patch.value, self.urids.atom.path, ())
    else {
      return;
    };
    let _ = path_value_writer.append(&self.file_path);
  }

  fn read_patch_set_events(
    &mut self,
    atom: UnidentifiedAtom<'static>,
    features: &mut AudioFeatures,
  ) {
    let mut should_read_patch_value = false;

    let (object_header, object_reader) = match atom.read(self.urids.atom.object, ()) {
      Some(object) => object,
      None => return,
    };

    if object_header.otype == self.urids.patch.set_class {
      for (property_header, property) in object_reader {
        if property_header.key == self.urids.patch.property {
          // A patch:Set for any other property carries a different atom type.
          should_read_patch_value = property
            .read(self.urids.atom.urid, ())
            .map(|patch_property| self.urids.sample.get() == patch_property.get())
            .unwrap_or(false);
        }

        if should_read_patch_value && property_header.key == self.urids.patch.value {
          let Some(path) = property.read(self.urids.atom.path, ()) else {
            continue;
          };
          self.file_path = path.to_string();

          features
            .schedule
            .schedule_work(WorkRequest::LoadFile(
              self.file_path.to_string(),
              self.sample_rate,
              self.time_warp.get_delay_line_size(),
            ))
            .ok();
        }
      }
    };
  }

  fn read_patch_get_events(&mut self, atom: UnidentifiedAtom<'static>, ports: &mut Ports) {
    let object_header = match atom.read(self.urids.atom.object, ()) {
      Some((object_header, _)) => object_header,
      None => return,
    };

    if object_header.otype == self.urids.patch.get_class && !self.file_path.is_empty() {
      self.write_set_file(ports);
    }
  }

  fn read_midi_events(&mut self, atom: UnidentifiedAtom<'static>) {
    let midi_message = match atom.read(self.urids.midi.wmidi, ()) {
      Some(midi_message) => midi_message,
      None => return,
    };

    match midi_message {
      MidiMessage::NoteOn(_, note, velocity) => {
        self
          .notes
          .note_on(note.into(), u8::from(velocity) as f32 / 127.);
      }
      MidiMessage::NoteOff(_, note, _) => {
        self.notes.note_off(note.into());
      }
      MidiMessage::ControlChange(_, cc, value) => match u8::from(cc) {
        64 => self.notes.sustain(u8::from(value) > 0),
        120 => self.notes.remove_notes(),
        123 => self.notes.release_notes(),
        _ => (),
      },
      MidiMessage::PitchBendChange(_, pitch_bend) => {
        let pitch_bend_factor = 2_f32.powf((u16::from(pitch_bend) as f32 - 8192.0) / 8192.0);
        self.params.set_pitch_bend_factor(pitch_bend_factor);
      }
      _ => (),
    };
  }
}
