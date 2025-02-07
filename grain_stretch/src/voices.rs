mod grain_trigger;
mod grains;
mod start_phasor;
use {
  crate::{notes::Note, shared::float_ext::FloatExt, stereo_delay_line::StereoDelayLine},
  grain_trigger::GrainTrigger,
  grains::Grains,
  start_phasor::StartPhasor,
};

pub struct Voices {
  grains: Vec<Grains>,
  grain_trigger: GrainTrigger,
  start_phasor: StartPhasor,
  fade_time: f32,
  sample_rate: f32,
}

impl Voices {
  pub fn new(sample_rate: f32, fade_time: f32) -> Self {
    Self {
      grains: vec![Grains::new(sample_rate); 8],
      grain_trigger: GrainTrigger::new(sample_rate),
      start_phasor: StartPhasor::new(sample_rate),
      fade_time,
      sample_rate,
    }
  }

  pub fn process(
    &mut self,
    delay_line: &StereoDelayLine,
    notes: &Vec<Note>,
    size: f32,
    time: f32,
    density: f32,
    speed: f32,
    stretch: f32,
    scan: f32,
    spray: f32,
    midi_enabled: bool,
  ) -> (f32, f32) {
    let duration = size.scale(0., 1., time, self.fade_time);
    let grain_density = density.scale(0., 1., 1., 15.);

    let trigger = self.grain_trigger.process(duration, grain_density);
    let start_phase = self
      .start_phasor
      .process(speed, time, size, density, stretch);

    let window_mode = (density - 1.).min(1.);
    let grain_duration = duration + self.fade_time * (1. - window_mode);
    let phase_step_size = grain_duration.mstosamps(self.sample_rate).recip();
    let window_factor = window_mode.scale(0., 1., grain_duration / self.fade_time, 2.);
    let fade_factor = time / self.fade_time;
    let fade_offset = fade_factor.recip() + 1.;

    if midi_enabled {
      notes
        .iter()
        .zip(self.grains.iter_mut())
        .fold((0., 0.), |result, (voice, grains)| {
          let grains_out = grains.process(
            delay_line,
            trigger,
            scan,
            spray,
            time,
            start_phase,
            phase_step_size,
            speed * voice.get_speed(),
            window_factor,
            fade_factor,
            fade_offset,
          );
          (
            result.0 + grains_out.0 * voice.get_gain(),
            result.1 + grains_out.1 * voice.get_gain(),
          )
        })
    } else {
      self.grains[0].process(
        delay_line,
        trigger,
        scan,
        spray,
        time,
        start_phase,
        phase_step_size,
        speed,
        window_factor,
        fade_factor,
        fade_offset,
      )
    }
  }
}
