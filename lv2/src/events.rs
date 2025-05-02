use {
  crate::{worker::WorkData, AudioFeatures, DmTimeWarp, Ports},
  lv2::{
    lv2_atom::atom_prelude::{AtomReadError, AtomWriteError},
    prelude::*,
  },
  std::ffi::CStr,
  wmidi::MidiMessage,
};

impl DmTimeWarp {
  pub fn handle_events(&mut self, ports: &mut Ports, features: &mut AudioFeatures) {
    let control_sequence = match ports
      .control
      .read(self.urids.atom.sequence)
      .and_then(|s| s.with_unit(self.urids.unit.frame))
    {
      Ok(sequence_iter) => sequence_iter,
      Err(_) => return,
    };

    let should_read_patch_value = false;
    for (time_stamp, atom) in control_sequence {
      self.time_stamp = time_stamp;
      self
        .read_patch_set_events(atom, features, should_read_patch_value)
        .ok();
      self.read_midi_events(atom).ok();
    }
  }

  pub fn write_set_file(
    &mut self,
    ports: &mut Ports,
    features: &mut AudioFeatures,
  ) -> Result<(), AtomWriteError> {
    let mut notify_sequence = ports
      .notify
      .write(self.urids.atom.sequence)
      .and_then(|s| s.with_unit(self.urids.unit.frame))?;

    let object_header = notify_sequence.new_event(self.time_stamp, self.urids.atom.object)?;
    let mut writer = object_header.write_header(ObjectHeader {
      id: None,
      otype: self.urids.patch.set_class.into_general(),
    })?;
    let mut property = writer.new_property(self.urids.sample, self.urids.atom.path)?;
    let message = format!("created new property for patch message\n\0");
    let _ = features.log.print_cstr(
      self.urids.log.note,
      CStr::from_bytes_with_nul(message.as_bytes()).unwrap(),
    );
    property.append(&self.file_path)?;
    let message = format!("patch set file path: {}\n\0", self.file_path);
    let _ = features.log.print_cstr(
      self.urids.log.note,
      CStr::from_bytes_with_nul(message.as_bytes()).unwrap(),
    );

    Ok(())
  }

  fn read_patch_set_events(
    &mut self,
    atom: &UnidentifiedAtom,
    features: &mut AudioFeatures,
    mut should_read_patch_value: bool,
  ) -> Result<(), AtomReadError> {
    let (object_header, object_reader) = atom.read(self.urids.atom.object)?;

    if object_header.otype == self.urids.patch.set_class {
      for (property_header, property) in object_reader {
        if property_header.key == self.urids.patch.property {
          should_read_patch_value = property
            .read(self.urids.atom.urid)
            .map(|patch_property| self.urids.sample.get() == patch_property.get())?;
        }

        if should_read_patch_value && property_header.key == self.urids.patch.value {
          self.file_path = property
            .read(self.urids.atom.path)
            .map(|path| path.to_string())?;

          features
            .schedule
            .schedule_work(WorkData::new(&self.file_path, self.sample_rate))
            .ok();
        }
      }
    }

    Ok(())
  }

  fn read_midi_events(&mut self, atom: &UnidentifiedAtom) -> Result<(), AtomReadError> {
    let midi_message = atom.read(self.urids.midi.wmidi)?;

    match midi_message {
      MidiMessage::NoteOn(_, note, velocity) => {
        self
          .notes
          .note_on(note.into(), (u8::from(velocity) / 127).into());
      }
      MidiMessage::NoteOff(_, note, _) => {
        self.notes.note_off(note.into());
      }
      _ => (),
    }

    Ok(())
  }
}
