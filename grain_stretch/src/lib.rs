mod filter;
mod grains;
mod params;
mod stereo_delay_line;
pub mod shared {
  pub mod delta;
  pub mod float_ext;
  pub mod phasor;
  pub mod tuple_ext;
}
use std::collections::HashMap;

pub use params::Params;
use {
  filter::Filter,
  grains::Grains,
  params::Smoother,
  shared::tuple_ext::TupleExt,
  stereo_delay_line::{Interpolation, StereoDelayLine},
};

pub const MIN_DELAY_TIME: f32 = 10.;
pub const MAX_DELAY_TIME: f32 = 10000.;

pub struct GrainStretch {
  delay_line: StereoDelayLine,
  grains: Grains,
  filter: Filter,
}

impl GrainStretch {
  const FADE_TIME: f32 = MIN_DELAY_TIME * 2.;

  pub fn new(sample_rate: f32) -> Self {
    Self {
      delay_line: StereoDelayLine::new(
        (sample_rate * (MAX_DELAY_TIME + Self::FADE_TIME) / 1000.) as usize,
        sample_rate,
      ),
      grains: Grains::new(sample_rate, Self::FADE_TIME),
      filter: Filter::new(sample_rate),
    }
  }

  pub fn process(&mut self, input: (f32, f32), params: &mut Params, note: Option<u8>) -> (f32, f32) {
    let Params {
      scan,
      spray,
      size,
      density,
      stretch,
      speed,
      midi_enabled,
      ..
    } = *params;
    let recording_gain = params.recording_gain.next();
    let time = params.time.next();
    let highpass = params.highpass.next();
    let lowpass = params.lowpass.next();
    let overdub = params.overdub.next();
    let recycle = params.recycle.next();
    let dry = params.dry.next();
    let wet = params.wet.next();

    let grains_out = self.grains.process(
        &self.delay_line,
        size,
        time,
        density,
        speed,
        stretch,
        scan,
        spray,
        midi_enabled,
        note,
      );

    self.write_to_delay(
      input,
      time,
      grains_out,
      overdub,
      recycle,
      recording_gain,
      highpass,
      lowpass,
    );
    input.multiply(dry).add(grains_out.multiply(wet))
  }

  fn write_to_delay(
    &mut self,
    input: (f32, f32),
    time: f32,
    grains_out: (f32, f32),
    overdub: f32,
    recycle: f32,
    recording_gain: f32,
    highpass: f32,
    lowpass: f32,
  ) {
    // Maybe instead of reading from the delay line to preserve buffer content, just stop continuously writing to the buffer
    let delay_out = self.delay_line.read(time, Interpolation::Linear);
    let feedback = self.get_feedback(delay_out, grains_out, recycle, overdub, highpass, lowpass);
    let delay_in =
      (input.add(feedback).multiply(recording_gain)).add(delay_out.multiply(1. - recording_gain));
    self.delay_line.write(delay_in);
  }

  fn get_feedback(
    &mut self,
    delay_out: (f32, f32),
    grains_out: (f32, f32),
    recycle: f32,
    overdub: f32,
    highpass: f32,
    lowpass: f32,
  ) -> (f32, f32) {
    let feedback = delay_out
      .multiply((1. - recycle) * overdub)
      .add(grains_out.multiply(recycle * overdub));
    let feedback = Self::clip(feedback);
    self.filter.process(feedback, highpass, lowpass)
  }

  fn clip(x: (f32, f32)) -> (f32, f32) {
    (x.0.clamp(-1., 1.), x.1.clamp(-1., 1.))
  }
}
