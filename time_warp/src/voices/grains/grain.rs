use {
  crate::{
    delay_line::{DelayLine, Interpolation},
    shared::float_ext::FloatExt,
  },
  std::f32::consts::FRAC_PI_2,
};

#[derive(Clone, Copy)]
pub struct Grain {
  phase: f32,
  position: f32,
  gain: (f32, f32),
  sample_factor: f32,
  is_active: bool,
}

impl Grain {
  pub fn new(sample_rate: f32) -> Self {
    Self {
      phase: 0.,
      position: 0.,
      gain: (0.5, 0.5),
      sample_factor: 1000. / sample_rate,
      is_active: false,
    }
  }

  pub fn process(
    &mut self,
    delay_line: &DelayLine,
    time: f32,
    phase_step_size: f32,
    speed: f32,
    window_factor: f32,
    fade_factor: f32,
    fade_offset: f32,
  ) -> (f32, f32, f32) {
    let position_a = Self::wrap(self.position) * 2.;
    let position_b = Self::wrap(self.position + 0.5) * 2.;
    let position_a_fade = Self::get_playhead_fade(position_a, fade_factor, fade_offset);
    let position_b_fade = 1. - position_a_fade;
    let grain_fade = self.get_grain_fade(window_factor);

    let next_phase = self.phase + phase_step_size;
    if next_phase < 1. {
      self.phase = next_phase;
    } else {
      self.is_active = false;
    }

    self.position += self.sample_factor / time * speed;
    let delay_out = Self::read_from_delay(
      delay_line,
      time,
      position_a,
      position_b,
      grain_fade,
      position_a_fade,
      position_b_fade,
    );
    (delay_out * self.gain.0, delay_out * self.gain.1, grain_fade)
  }

  pub fn reset(&mut self) {
    self.phase = 0.;
    self.position = 0.;
    self.is_active = false;
  }

  pub fn set_parameters(
    &mut self,
    scan: f32,
    spray: f32,
    stereo: f32,
    time: f32,
    start_position_phase: f32,
  ) {
    let spray = fastrand::f32() * spray / time;

    self.phase = 0.;
    self.position = 1. - (scan + spray + start_position_phase).fract() * 0.5;
    self.is_active = true;
    self.set_panning(stereo);
  }

  fn read_from_delay(
    delay_line: &DelayLine,
    time: f32,
    position_a: f32,
    position_b: f32,
    grain_fade: f32,
    position_a_fade: f32,
    position_b_fade: f32,
  ) -> f32 {
    match (position_a_fade > 0., position_b_fade > 0.) {
      (true, true) => {
        delay_line
        .read(position_a * time, Interpolation::Linear)
         * position_a_fade.min(grain_fade) // take the minimum of both fades to prevent audible decreasing gain
         + delay_line
            .read(position_b * time, Interpolation::Linear)
            * position_b_fade.min(grain_fade)
      }
      (true, false) => {
        delay_line.read(position_a * time, Interpolation::Linear) * position_a_fade.min(grain_fade)
      }
      (false, true) => {
        delay_line.read(position_b * time, Interpolation::Linear) * position_b_fade.min(grain_fade)
      }
      _ => 0.,
    }
  }

  pub fn is_active(&self) -> bool {
    self.is_active
  }

  fn get_grain_fade(&self, window_factor: f32) -> f32 {
    let fade_in = (self.phase * window_factor).min(1.);
    let fade_out = ((1. - self.phase) * window_factor).min(1.);
    let fade = fade_in.min(fade_out);
    Self::apply_curve(fade)
  }

  fn get_playhead_fade(position: f32, fade_factor: f32, fade_offset: f32) -> f32 {
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
    if x < 0. {
      x + 1.
    } else {
      x.fract()
    }
  }

  fn set_panning(&mut self, stereo: f32) {
    if stereo == 0. {
      self.gain = (0.5, 0.5);
      return;
    }
    if stereo == 1. {
      self.gain = if fastrand::bool() { (1., 0.) } else { (0., 1.) };
      return;
    }

    if stereo > 0.8 {
      let stereo_factor = (stereo - 0.8) * 2.5;
      let hard_panning = if fastrand::bool() { 1. } else { 0. };
      let random_panning = (fastrand::f32() - 0.5) + 0.5;
      let panning = random_panning.mix(hard_panning, stereo_factor);
      self.gain = (panning, 1. - panning)
    } else {
      let stereo_factor = stereo * 1.25;
      let panning = (fastrand::f32() - 0.5) * stereo_factor + 0.5;
      self.gain = (panning, 1. - panning);
    }
  }
}
