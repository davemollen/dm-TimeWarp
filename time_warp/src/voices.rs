mod grain_trigger;
mod grains;
mod linear_adsr;
mod start_phasor;
use {
  crate::{
    notes::{ADSRStage, Note},
    shared::float_ext::FloatExt,
    stereo_delay_line::StereoDelayLine,
  },
  grain_trigger::GrainTrigger,
  grains::Grains,
  linear_adsr::ADSR,
  start_phasor::StartPhasor,
};

pub struct Voices {
  grains: Vec<Grains>,
  adsr: Vec<ADSR>,
  grain_trigger: GrainTrigger,
  start_phasor: StartPhasor,
  fade_time: f32,
  sample_rate: f32,
}

impl Voices {
  pub fn new(sample_rate: f32, fade_time: f32) -> Self {
    Self {
      grains: vec![Grains::new(sample_rate); 8],
      adsr: vec![ADSR::new(sample_rate, 5.); 8],
      grain_trigger: GrainTrigger::new(sample_rate),
      start_phasor: StartPhasor::new(sample_rate),
      fade_time,
      sample_rate,
    }
  }

  pub fn process(
    &mut self,
    delay_line: &StereoDelayLine,
    notes: &mut Vec<Note>,
    size: f32,
    time: f32,
    density: f32,
    speed: f32,
    stretch: f32,
    scan: f32,
    spray: f32,
    midi_enabled: bool,
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    is_recording: bool,
    reset_playback: bool,
  ) -> (f32, f32) {
    let duration = (1. - size) * (time - self.fade_time) + self.fade_time; // range from time to fade_time
    let grain_density = density * 14. + 1.; // range from 1 to 15

    if reset_playback {
      self.start_phasor.reset();
    }
    let start_phase = self
      .start_phasor
      .process(speed, time, size, density, stretch, is_recording);

    let window_mode = (density - 1.).min(1.);
    let grain_duration = duration + self.fade_time * (1. - window_mode);
    let phase_step_size = grain_duration.mstosamps(self.sample_rate).recip();
    let window_factor = window_mode.scale(0., 1., grain_duration / self.fade_time, 2.);
    let fade_factor = time / self.fade_time;
    let fade_offset = fade_factor.recip() + 1.;

    if midi_enabled {
      notes
        .iter_mut()
        .filter(|n| *n.get_adsr_stage() != ADSRStage::Idle)
        .zip(self.grains.iter_mut())
        .zip(self.adsr.iter_mut())
        .fold((0., 0.), |result, ((note, grains), adsr)| {
          let gain = adsr.process(note, attack, decay, sustain, release);
          if reset_playback {
            grains.reset();
          }
          let trigger = self.grain_trigger.process(
            duration,
            grain_density,
            adsr.get_trigger() || reset_playback,
          );

          let grains_out = grains.process(
            delay_line,
            trigger,
            scan,
            spray,
            time,
            start_phase,
            phase_step_size,
            speed * adsr.get_speed(),
            window_factor,
            fade_factor,
            fade_offset,
            is_recording,
          );
          (
            result.0 + grains_out.0 * gain,
            result.1 + grains_out.1 * gain,
          )
        })
    } else {
      if reset_playback {
        self.grains[0].reset();
      }
      let trigger = self
        .grain_trigger
        .process(duration, grain_density, reset_playback);
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
        is_recording,
      )
    }
  }
}
