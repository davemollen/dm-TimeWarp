use {
  crate::{
    shared::{float_ext::FloatExt, tuple_ext::TupleExt},
    stereo_delay_line::StereoDelayLine,
  },
  grain::Grain,
  grain_trigger::GrainTrigger,
  start_phasor::StartPhasor,
};

mod grain;
mod grain_trigger;
mod start_phasor;

pub struct Grains {
  grain_trigger: GrainTrigger,
  start_phasor: StartPhasor,
  grains: Vec<Grain>,
  fade_time: f32,
  sample_rate: f32,
}

impl Grains {
  pub fn new(sample_rate: f32, fade_time: f32) -> Self {
    Self {
      grain_trigger: GrainTrigger::new(sample_rate),
      start_phasor: StartPhasor::new(sample_rate),
      grains: vec![Grain::new(sample_rate); 20],
      fade_time,
      sample_rate,
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
  ) -> (f32, f32) {
    let duration = size.scale(0., 1., time, self.fade_time);
    let grain_density = density.scale(0., 1., 1., 15.);

    let trigger = self.grain_trigger.process(duration, grain_density);
    let start_phase = self
      .start_phasor
      .process(speed, time, size, density, stretch);

    let grain_speed = (1. - speed) * 0.5;
    let window_mode = (grain_density - 1.).min(1.);
    let grain_duration = duration + self.fade_time * (1. - window_mode);
    let phase_step_size = grain_duration.mstosamps(self.sample_rate).recip();
    let window_factor = window_mode.scale(0., 1., grain_duration / self.fade_time, 2.);
    let fade_factor = time / self.fade_time;
    let fade_offset = fade_factor.recip() + 1.;

    if trigger {
      let inactive_grain = self.grains.iter_mut().find(|grain| !grain.is_active());
      match inactive_grain {
        Some(grain) => grain.set_parameters(scan, spray, time, start_phase),
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
            delay_line,
            time,
            phase_step_size,
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

    (grains_left, grain_right).multiply(if gain == 0. { 0. } else { gain.recip() })
  }
}
