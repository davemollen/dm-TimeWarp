mod filter;
mod grain;
mod params;
mod start_phasor;
mod stereo_delay_line;
pub mod shared {
  pub mod delta;
  pub mod float_ext;
  pub mod phasor;
  pub mod tuple_ext;
}
use grain::Grain;
pub use params::Params;
use start_phasor::StartPhasor;
use stereo_delay_line::Interpolation;
use {
  filter::Filter,
  params::Smoother,
  shared::{delta::Delta, float_ext::FloatExt, phasor::Phasor, tuple_ext::TupleExt},
  stereo_delay_line::StereoDelayLine,
};

pub struct GrainStretch {
  fade_time: f32,
  sample_rate: f32,
  trigger_phasor: Phasor,
  trigger_delta: Delta,
  start_phasor: StartPhasor,
  delay_line: StereoDelayLine,
  filter: Filter,
  grains: Vec<Grain>,
}

impl GrainStretch {
  pub fn new(sample_rate: f32) -> Self {
    Self {
      fade_time: 20_f32.mstosamps(sample_rate),
      sample_rate,
      trigger_phasor: Phasor::new(sample_rate),
      trigger_delta: Delta::new(),
      start_phasor: StartPhasor::new(sample_rate),
      delay_line: StereoDelayLine::new((sample_rate * 10.01) as usize, sample_rate),
      filter: Filter::new(sample_rate),
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
    let trigger_phase = self
      .trigger_phasor
      .process(self.sample_rate / duration * grain_density);
    let trigger = self.trigger_delta.process(trigger_phase) < 0.;
    let grain_speed = (1. - speed) * 0.5;

    let start_phase = self
      .start_phasor
      .process(speed, time_in_samples, size, density, stretch)
      + scan;

    let window_mode = (density - 1.).min(1.);
    let duration = duration + self.fade_time * (1. - window_mode);
    let window_factor = window_mode.scale(0., 1., duration / self.fade_time, 2.);
    let fade_factor = time_in_samples / self.fade_time;
    let fade_offset = (fade_factor + 1.).recip();

    if trigger {
      let inactive_grain = self.grains.iter_mut().find(|grain| !grain.is_active());
      match inactive_grain {
        Some(grain) => grain.set_parameters(spray, start_phase),
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
            &mut self.delay_line,
            time,
            duration,
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

    // let feedback = self
    //   .delay_line
    //   .read(time, Interpolation::Linear)
    //   .multiply((1. - recycle) * overdub)
    //   .add(grains_out.multiply(recycle * overdub));

    // // Maybe instead of reading from the delay line to preserve buffer content, just stop continuously writing to the buffer
    // let preserved = self.delay_line.read(time, Interpolation::Linear);
    // self.delay_line.write(
    //   input
    //     .add(self.filter.process(feedback, highpass, lowpass))
    //     .multiply(recording_gain)
    //     .add(preserved.multiply(1. - recording_gain)),
    // );

    self.delay_line.write(input);
    input.multiply(dry).add(grains_out.multiply(wet))
  }
}
