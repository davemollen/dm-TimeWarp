use crate::{
  delay_line::{DelayLine, Interpolation},
  shared::float_ext::FloatExt,
};

#[derive(Clone, Copy)]
pub struct Grain {
  phase: f64,
  position: f64,
  gain: (f32, f32),
  sample_factor: f64,
  is_reversed: bool,
  is_active: bool,
}

impl Grain {
  pub fn new(sample_rate: f32) -> Self {
    Self {
      phase: 0.,
      position: 0.,
      gain: (0.5, 0.5),
      sample_factor: 1000. / sample_rate as f64,
      is_reversed: false,
      is_active: false,
    }
  }

  pub fn process(
    &mut self,
    delay_line: &DelayLine,
    time: f64,
    phase_step_size: f64,
    speed: f64,
    window_factor: f64,
  ) -> (f32, f32, f32) {
    let speed = if self.is_reversed {
      1. + speed
    } else {
      1. - speed
    };

    let grain_fade = self.get_grain_fade(window_factor);
    let delay_out = delay_line.read(
      (self.position * time) as f32 + time as f32,
      Interpolation::Linear,
    ) * grain_fade;
    self.position = Self::wrap(self.position + self.sample_factor / time * speed);
    let next_phase = self.phase + phase_step_size;
    if next_phase < 1. {
      self.phase = next_phase;
    } else {
      self.is_active = false;
    }
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
    is_reversed: bool,
  ) {
    let spray = fastrand::f32() * spray / time;

    self.phase = 0.;
    self.position = (1. - (scan + spray + start_position_phase).fract()) as f64;
    self.is_active = true;
    self.is_reversed = is_reversed;
    self.set_panning(stereo);
  }

  pub fn is_active(&self) -> bool {
    self.is_active
  }

  fn get_grain_fade(&self, window_factor: f64) -> f32 {
    let fade_in = (self.phase * window_factor).min(1.);
    let fade_out = ((1. - self.phase) * window_factor).min(1.);
    let fade = fade_in * fade_out;
    fade.cubic_spline_curve() as f32
  }

  fn wrap(x: f64) -> f64 {
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
