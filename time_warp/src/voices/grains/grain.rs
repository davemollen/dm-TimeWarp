use {
  crate::{
    shared::{float_ext::FloatExt, tuple_ext::TupleExt},
    stereo_delay_line::{Interpolation, StereoDelayLine},
  },
  std::f32::consts::FRAC_PI_2,
};

#[derive(Clone, Copy)]
pub struct Grain {
  phase: f32,
  position: f32,
  sample_factor: f32,
  is_active: bool,
}

impl Grain {
  pub fn new(sample_rate: f32) -> Self {
    Self {
      phase: 0.,
      position: 0.,
      sample_factor: 1000. / sample_rate,
      is_active: false,
    }
  }

  pub fn process(
    &mut self,
    delay_line: &StereoDelayLine,
    time: f32,
    phase_step_size: f32,
    speed: f32,
    window_factor: f32,
    fade_factor: f32,
    fade_offset: f32,
  ) -> (f32, f32, f32) {
    let position_a = Self::wrap(self.position) * 2.;
    let position_b = Self::wrap(self.position + 0.5) * 2.;
    // TODO: investigate if there's double windowing going on
    let grain_fade = self.get_grain_fade(window_factor);
    let position_a_fade = Self::get_xfade(position_a, fade_factor, fade_offset);
    let position_b_fade = 1. - position_a_fade;

    let next_phase = self.phase + phase_step_size;
    if next_phase < 1. {
      self.phase = next_phase;
    } else {
      self.is_active = false;
    }
    self.position = self.position + self.sample_factor / time * speed;

    let delay_out = Self::read_from_delay(
      delay_line,
      time,
      position_a,
      position_b,
      grain_fade,
      position_a_fade,
      position_b_fade,
    );

    (delay_out.0, delay_out.1, grain_fade)
  }

  pub fn reset(&mut self) {
    self.phase = 0.;
    self.position = 0.;
    self.is_active = false;
  }

  pub fn set_parameters(&mut self, scan: f32, spray: f32, time: f32, start_phase: f32) {
    let spray = fastrand::f32() * spray / time;

    self.phase = 0.;
    self.position = (1. - (scan + spray + start_phase).fract()) * 0.5;
    self.is_active = true;
  }

  pub fn is_active(&self) -> bool {
    self.is_active
  }

  fn read_from_delay(
    delay_line: &StereoDelayLine,
    time: f32,
    position_a: f32,
    position_b: f32,
    grain_fade: f32,
    position_a_fade: f32,
    position_b_fade: f32,
  ) -> (f32, f32) {
    match (position_a_fade > 0., position_b_fade > 0.) {
      (true, true) => delay_line
        .read(position_a * time, Interpolation::Linear)
        .multiply(position_a_fade * grain_fade)
        .add(
          delay_line
            .read(position_b * time, Interpolation::Linear)
            .multiply(position_b_fade * grain_fade),
        ),
      (true, false) => delay_line
        .read(position_a * time, Interpolation::Linear)
        .multiply(position_a_fade * grain_fade),
      (false, true) => delay_line
        .read(position_b * time, Interpolation::Linear)
        .multiply(position_b_fade * grain_fade),
      _ => (0., 0.),
    }
  }

  fn get_grain_fade(&self, window_factor: f32) -> f32 {
    let fade_in = (self.phase * window_factor).min(1.);
    let fade_out = ((1. - self.phase) * window_factor).min(1.);
    let fade = fade_in * fade_out;
    Self::apply_curve(fade)
  }

  fn get_xfade(position: f32, fade_factor: f32, fade_offset: f32) -> f32 {
    let fade =
      (position * fade_factor).min(1.) * ((fade_offset - position) * fade_factor).clamp(0., 1.);
    Self::apply_curve(fade)
  }

  fn apply_curve(x: f32) -> f32 {
    if x == 0. {
      0.
    } else if x == 1. {
      1.
    } else {
      let y = (x * FRAC_PI_2).fast_sin_bhaskara();
      y * y
    }
  }

  fn wrap(x: f32) -> f32 {
    if x <= 0. {
      x + 1.
    } else {
      x.fract()
    }
  }
}
