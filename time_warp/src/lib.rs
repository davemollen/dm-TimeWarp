mod delay_line;
mod filter;
mod mix;
mod notes;
mod params;
mod voices;
pub mod shared {
  pub mod delta;
  pub mod float_ext;
  pub mod phasor;
  pub mod tuple_ext;
}
mod audio_file_processor;

pub use {
  audio_file_processor::{AudioFileData, AudioFileProcessor},
  delay_line::{DelayLine, Interpolation},
  notes::Notes,
  params::{Params, SampleMode},
};
use {
  filter::Filter, mix::Mix, notes::Note, params::Smoother, shared::float_ext::FloatExt,
  shared::tuple_ext::TupleExt, voices::Voices,
};

const FADE_TIME: f32 = 5.;
pub const MIN_DELAY_TIME: f32 = 10.; // double of FADE_TIME
const MAX_DELAY_TIME: f32 = 60000.;
pub const MIN_DENSITY: f32 = 1.;
pub const MAX_DENSITY: f32 = 8.;
pub const CENTER_GRAIN_DURATION: f32 = 500.;

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
        (sample_rate * (MAX_DELAY_TIME + FADE_TIME) / 1000.) as usize,
        sample_rate,
      ),
      voices: Voices::new(sample_rate),
      filter: Filter::new(sample_rate),
      mix: Mix::new(),
    }
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
      density,
      stereo,
      speed,
      stretch,
      midi_enabled,
      sync_position,
      reset_playback,
      start_offset_phase,
      ..
    } = *params;

    let recording_gain = params.recording_gain.next();
    let playback_gain = params.playback_gain.next();
    let time = params.time.next();
    let recycle = params.recycle.next();
    let feedback = params.feedback.next();
    let dry = params.dry.next();
    let wet = params.wet.next();
    let attack = params.attack.next();
    let decay = params.decay.next();
    let sustain = params.sustain.next();
    let release = params.release.next();

    let grains_out = self
      .voices
      .process(
        &self.delay_line,
        notes,
        size,
        time,
        density,
        stereo,
        speed,
        stretch,
        scan,
        spray,
        midi_enabled,
        sync_position,
        attack,
        decay,
        sustain,
        release,
        reset_playback,
        start_offset_phase,
      )
      .multiply(playback_gain);
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
