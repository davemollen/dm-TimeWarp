mod delay_line;
mod filter;
mod mix;
mod notes;
mod params;
mod voices;
pub mod shared {
  pub mod delta;
  pub mod float_ext;
  pub mod macros;
  pub mod phasor;
  pub mod tuple_ext;
}
mod audio_file_processor;

use {
  crate::shared::tuple_ext::TupleExt, filter::Filter, mix::Mix, notes::Note, params::Smoother,
  shared::float_ext::FloatExt, voices::Voices,
};
pub use {
  audio_file_processor::{AudioFileData, AudioFileProcessor},
  delay_line::{DelayLine, Interpolation},
  notes::Notes,
  params::{Params, SampleMode},
};

const FADE_TIME: f64 = 5.;
pub const MIN_DELAY_TIME: f32 = 10.; // double of FADE_TIME
const MAX_DELAY_TIME: f32 = 60000.;
pub const MIN_DENSITY: f64 = 1.;
pub const MAX_DENSITY: f64 = 8.;
pub const CENTER_GRAIN_DURATION: f32 = 500.;
pub const MAX_VOICE_COUNT: usize = 8;

pub struct TimeWarp {
  delay_line: DelayLine,
  voices: Voices,
  filter: Filter,
  mix: Mix,
}

impl TimeWarp {
  pub fn new(sample_rate: f32) -> Self {
    Self {
      delay_line: DelayLine::new(
        (sample_rate * (MAX_DELAY_TIME + FADE_TIME as f32) / 1000.) as usize,
        sample_rate,
      ),
      voices: Voices::new(sample_rate),
      filter: Filter::new(sample_rate),
      mix: Mix::new(),
    }
  }

  pub fn reset(&mut self) {
    self.filter.reset();
    self.voices.reset();
  }

  pub fn reset_delay_line(&mut self) {
    self.delay_line.reset();
  }

  pub fn process(
    &mut self,
    input: (f32, f32),
    params: &mut Params,
    notes: &mut Vec<Note>,
  ) -> (f32, f32) {
    let Params {
      scan,
      spray,
      size,
      stereo,
      speed,
      stretch,
      midi_enabled,
      sync_position,
      should_reset_playback,
      start_offset_phase,
      ..
    } = *params;

    let density = params.density.next();
    let recording_gain = params.recording_gain.next();
    let playback_gain = params.playback_gain.next();
    // Length can drive the target to 0 ms, and 1000. / time then latches inf/NaN
    // into the grain trigger and start position phasors, silencing the plugin.
    let time = params.time.next().max(MIN_DELAY_TIME);
    let recycle = params.recycle.next();
    let feedback = params.feedback.next();
    let dry = params.dry.next();
    let wet = params.wet.next();
    let attack = params.attack.next();
    let decay = params.decay.next();
    let sustain = params.sustain.next();
    let release = params.release.next();

    let (grains_out, grains_gain) = self.voices.process(
      &self.delay_line,
      notes,
      size,
      time,
      density as f64,
      stereo,
      speed as f64,
      stretch as f64,
      scan,
      spray,
      midi_enabled,
      sync_position,
      attack,
      decay,
      sustain,
      release,
      should_reset_playback,
      start_offset_phase,
    );
    let gain_compensation = if grains_gain == 0. {
      0.
    } else {
      grains_gain.recip().sqrt()
    };
    let grains_out = grains_out.multiply(playback_gain * gain_compensation);
    self.write_to_delay(input, time, grains_out, recycle, feedback, recording_gain);
    let output = input.multiply(dry).add(grains_out.multiply(wet));
    params.settle();

    output
  }

  pub fn get_delay_line_size(&self) -> usize {
    self.delay_line.get_size()
  }

  pub fn set_delay_line_values(&mut self, values: Vec<f32>, write_pointer_index: usize) {
    self.delay_line.set_values(values);
    self.delay_line.set_write_pointer(write_pointer_index);
  }

  pub fn get_filter(&mut self) -> &mut Filter {
    &mut self.filter
  }

  fn write_to_delay(
    &mut self,
    input: (f32, f32),
    time: f32,
    grains_out: (f32, f32),
    recycle: f32,
    feedback: f32,
    recording_gain: f32,
  ) {
    let input = input.0 + input.1;
    let grains_out = grains_out.0 + grains_out.1;
    let delay_out = self.delay_line.read(time, Interpolation::Linear);
    let feedback = self.get_feedback(delay_out, grains_out, recycle, feedback);
    let delay_in = self
      .mix
      .process(delay_out, input + feedback, recording_gain);
    self.delay_line.write(delay_in);
  }

  fn get_feedback(&mut self, delay_out: f32, grains_out: f32, recycle: f32, feedback: f32) -> f32 {
    if feedback == 0. {
      return 0.;
    }
    let feedback_signal = delay_out.mix(grains_out, recycle) * feedback;
    self.filter.process(feedback_signal.clamp(-1., 1.))
  }
}

#[cfg(test)]
mod tests {
  use super::{Notes, Params, SampleMode, TimeWarp};
  use std::f32::consts::TAU;

  /// Pitch and Detune at their maxima give an effective speed of 2^(25/12) = 4.24,
  /// which moves a grain's position below -1. within its lifetime. A position that
  /// far negative used to reach get_playhead_fade unnormalized, where it turned
  /// into a large negative gain on one of the two crossfade taps: this measured
  /// 83.6 with a 0.5 amplitude input, and 11602 at a Time of 500 ms.
  #[test]
  fn should_not_amplify_the_input_at_maximum_pitch() {
    let sample_rate = 48000.;
    let buffer_size = 128;
    let mut time_warp = TimeWarp::new(sample_rate);
    let mut params = Params::new(sample_rate);
    let mut notes = Notes::new();
    let mut phase = 0.;
    let mut peak = 0_f32;

    for _ in 0..(sample_rate as usize / buffer_size / 4) {
      params.set(
        true,              // record
        true,              // play
        false,             // erase
        0.,                // scan
        0.,                // spray, kept at 0. so the grain positions are deterministic
        false,             // freeze
        1.,                // stretch
        1.,                // size
        1.,                // density
        0.,                // stereo, kept at 0. so the panning is deterministic
        100.,              // detune
        24.,               // pitch
        SampleMode::Delay, //
        20.,               // time
        1.,                // length
        0.,                // recycle
        0.,                // feedback
        1.,                // attack
        5.,                // decay
        1.,                // sustain
        5.,                // release
        false,             // midi_enabled
        false,             // sync_position
        -70.,              // dry, muted so only the grains are measured
        0.,                // wet
        buffer_size,
      );
      for _ in 0..buffer_size {
        phase = (phase + 1000. / sample_rate).fract();
        let input = (phase * TAU).sin() * 0.5;
        let (left, right) = time_warp.process((input, input), &mut params, notes.get_notes());
        peak = peak.max(left.abs()).max(right.abs());
      }
    }

    assert!(
      peak < 2.,
      "the loudest sample was {peak}, which is more than the 0.5 amplitude input"
    );
  }
}
