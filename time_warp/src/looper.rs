use crate::{
  shared::{delta::Delta, phasor::Phasor},
  DelayLine,
};

pub struct Looper {
  phasor: Phasor,
  delta: Delta,
  sample_rate: f32,
}

impl Looper {
  pub fn new(sample_rate: f32) -> Self {
    Self {
      phasor: Phasor::new(sample_rate),
      delta: Delta::new(),
      sample_rate,
    }
  }

  pub fn process(
    &mut self,
    input: f32,
    delay_line: &mut DelayLine,
    loop_duration: Option<f32>,
    recording_gain: f32,
    feedback: f32,
  ) {
    match loop_duration {
      Some(time) => {
        let phase = self.phasor.process(1000. / time);
        // self.delta.process(phase);

        let time_in_ms = phase * time;

        // let read = delay_line.loop_read(time_in_ms, time);
        if recording_gain > 0. {
          delay_line.loop_write(time_in_ms, input * recording_gain, time);
        }
      }
      None => {
        if recording_gain > 0. {
          delay_line.write(input * recording_gain);
        }
      }
    }
  }

  pub fn reset(&mut self) {
    self.phasor.reset();
  }
}
