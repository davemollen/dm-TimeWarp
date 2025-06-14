mod filter;
mod mix;
mod notes;
mod params;
mod stereo_delay_line;
mod voices;
pub mod shared {
  pub mod delta;
  pub mod float_ext;
  pub mod phasor;
  pub mod tuple_ext;
}
mod wav_processor;

use {
  filter::Filter, mix::Mix, notes::Note, params::Smoother, shared::tuple_ext::TupleExt,
  voices::Voices,
};
pub use {
  notes::Notes,
  params::{Params, RecordMode},
  stereo_delay_line::{Interpolation, StereoDelayLine},
  wav_processor::{WavFileData, WavProcessor},
};

pub const MIN_DELAY_TIME: f32 = 2.5;
const MAX_DELAY_TIME: f32 = 60000.;

pub struct TimeWarp {
  delay_line: StereoDelayLine,
  voices: Voices,
  filter: Filter,
  mix: Mix,
}

impl TimeWarp {
  const FADE_TIME: f32 = MIN_DELAY_TIME * 2.;

  pub fn new(sample_rate: f32) -> Self {
    Self {
      delay_line: StereoDelayLine::new(
        (sample_rate * (MAX_DELAY_TIME + Self::FADE_TIME) / 1000.) as usize,
        sample_rate,
      ),
      voices: Voices::new(sample_rate, Self::FADE_TIME),
      filter: Filter::new(sample_rate),
      mix: Mix::default(),
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
      stretch,
      speed,
      midi_enabled,
      reset_playback,
      ..
    } = *params;

    let recording_gain = params.recording_gain.next();
    let playback_gain = params.playback_gain.next();
    let time = params.time.next();
    let feedback = params.feedback.next();
    let recycle = params.recycle.next();
    let dry = params.dry.next();
    let wet = params.wet.next();
    let attack = params.attack.next();
    let decay = params.decay.next();
    let sustain = params.sustain.next();
    let release = params.release.next();
    let is_recording = recording_gain > 0.;

    let grains_out = self
      .voices
      .process(
        &self.delay_line,
        notes,
        size,
        time,
        density,
        speed,
        stretch,
        scan,
        spray,
        midi_enabled,
        attack,
        decay,
        sustain,
        release,
        is_recording,
        reset_playback,
      )
      .multiply(playback_gain);

    self.write_to_delay(
      input,
      time,
      grains_out,
      feedback,
      recycle,
      recording_gain,
      is_recording,
    );

    input.multiply(dry).add(grains_out.multiply(wet))
  }

  pub fn get_delay_line_size(&self) -> usize {
    self.delay_line.get_size()
  }

  pub fn set_delay_line_values(&mut self, values: Vec<(f32, f32)>, write_pointer_index: usize) {
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
    feedback: f32,
    recycle: f32,
    recording_gain: f32,
    is_recording: bool,
  ) {
    if is_recording {
      let delay_out = self.delay_line.read(time, Interpolation::Linear);
      let feedback = self.get_feedback(delay_out, grains_out, recycle, feedback);
      let delay_in = self
        .mix
        .process(delay_out, input.add(feedback), recording_gain);
      self.delay_line.write(delay_in);
    }
  }

  fn get_feedback(
    &mut self,
    delay_out: (f32, f32),
    grains_out: (f32, f32),
    recycle: f32,
    feedback: f32,
  ) -> (f32, f32) {
    let feedback = delay_out
      .multiply((1. - recycle) * feedback)
      .add(grains_out.multiply(recycle * feedback));
    let feedback = Self::clip(feedback);
    self.filter.process(feedback)
  }

  fn clip(x: (f32, f32)) -> (f32, f32) {
    (x.0.clamp(-1., 1.), x.1.clamp(-1., 1.))
  }
}
