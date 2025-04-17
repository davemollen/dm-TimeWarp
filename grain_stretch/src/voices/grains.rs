mod grain;
use {
  crate::{shared::tuple_ext::TupleExt, stereo_delay_line::StereoDelayLine},
  grain::Grain,
};

#[derive(Clone)]
pub struct Grains {
  grains: Vec<Grain>,
}

impl Grains {
  pub fn new(sample_rate: f32) -> Self {
    Self {
      grains: vec![Grain::new(sample_rate); 20],
    }
  }

  pub fn process(
    &mut self,
    delay_line: &StereoDelayLine,
    trigger: bool,
    scan: f32,
    spray: f32,
    time: f32,
    start_phase: f32,
    phase_step_size: f32,
    speed: f32,
    window_factor: f32,
    fade_factor: f32,
    fade_offset: f32,
    is_recording: bool,
  ) -> (f32, f32) {
    let speed = if is_recording {
      (1. - speed) * 0.5
    } else {
      speed * -0.5
    };
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
            speed,
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

    (grains_left, grain_right).multiply(if gain == 0. { 0. } else { gain.recip().sqrt() })
  }

  pub fn reset(&mut self) {
    self.grains.iter_mut().for_each(|grain| grain.reset());
  }
}
