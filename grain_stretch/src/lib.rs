mod dc_block;
mod filter;
mod grain;
mod grain_trigger;
mod params;
mod start_phasor;
mod stereo_delay_line;
pub mod shared {
  pub mod delta;
  pub mod float_ext;
  pub mod phasor;
  pub mod tuple_ext;
}
pub use params::Params;
use {
  dc_block::DcBlock,
  filter::Filter,
  grain::Grain,
  grain_trigger::GrainTrigger,
  params::Smoother,
  shared::{float_ext::FloatExt, tuple_ext::TupleExt},
  start_phasor::StartPhasor,
  stereo_delay_line::{Interpolation, StereoDelayLine},
};

pub const MIN_DELAY_TIME: f32 = 10.;
pub const MAX_DELAY_TIME: f32 = 10000.;

pub struct GrainStretch {
  fade_time: f32,
  sample_rate: f32,
  grain_trigger: GrainTrigger,
  start_phasor: StartPhasor,
  delay_line: StereoDelayLine,
  filter: Filter,
  dc_block: DcBlock,
  grains: Vec<Grain>,
}

impl GrainStretch {
  const FADE_TIME: f32 = MIN_DELAY_TIME * 2.;

  pub fn new(sample_rate: f32) -> Self {
    Self {
      fade_time: Self::FADE_TIME.mstosamps(sample_rate),
      sample_rate,
      grain_trigger: GrainTrigger::new(sample_rate),
      start_phasor: StartPhasor::new(sample_rate),
      delay_line: StereoDelayLine::new(
        (sample_rate * (MAX_DELAY_TIME + Self::FADE_TIME) / 1000.) as usize,
        sample_rate,
      ),
      filter: Filter::new(sample_rate),
      dc_block: DcBlock::new(sample_rate),
      grains: vec![Grain::new(sample_rate); 20],
    }
  }

  pub fn process(&mut self, input: (f32, f32), params: &mut Params) -> (f32, f32) {
    let Params {
      scan,
      spray,
      size,
      density,
      stretch,
      speed,
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

    let time_in_samples = time.mstosamps(self.sample_rate);
    let duration = size.scale(0., 1., time_in_samples, self.fade_time);
    let grain_density = density.scale(0., 1., 1., 15.);

    let trigger = self.grain_trigger.process(duration, grain_density);
    let start_phase = self
      .start_phasor
      .process(speed, time_in_samples, size, density, stretch);

    let grain_speed = (1. - speed) * 0.5;
    let window_mode = (grain_density - 1.).min(1.);
    let grain_duration = duration + self.fade_time * (1. - window_mode);
    let window_factor = window_mode.scale(0., 1., grain_duration / self.fade_time, 2.);
    let fade_factor = time_in_samples / self.fade_time;
    let fade_offset = fade_factor.recip() + 1.;

    if trigger {
      let inactive_grain = self.grains.iter_mut().find(|grain| !grain.is_active());
      match inactive_grain {
        Some(grain) => grain.set_parameters(scan, spray, start_phase),
        _ => {}
      }
    }

    let (grains_left, grain_right, gain) = self
      .grains
      .iter_mut()
      .filter(|grain| grain.is_active())
      .fold(
        (0., 0., 0.),
        |(left_output, right_output, acc_gain), grain| {
          let (left_grain, right_grain, grain_gain) = grain.process(
            &self.delay_line,
            time,
            grain_duration,
            grain_speed,
            window_factor,
            fade_factor,
            fade_offset,
          );
          (
            left_output + left_grain,
            right_output + right_grain,
            acc_gain + grain_gain,
          )
        },
      );
    let grains_out = (grains_left, grain_right).multiply(gain.recip());

    self.write_to_delay(input, time, grains_out, overdub, recycle, recording_gain);
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
  ) {
    // Maybe instead of reading from the delay line to preserve buffer content, just stop continuously writing to the buffer
    let delay_out = self.delay_line.read(time, Interpolation::Linear);
    let feedback = delay_out.multiply((1. - recycle) * overdub);
    // .add(grains_out.multiply(recycle * overdub));
    // let filtered_feedback = self.filter.process(feedback, highpass, lowpass);
    let delay_in =
      (input.add(feedback).multiply(recording_gain)).add(delay_out.multiply(1. - recording_gain));
    self.delay_line.write(delay_in);
  }
}
