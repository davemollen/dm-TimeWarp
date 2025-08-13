mod grain_trigger;
mod grains;
mod linear_adsr;
use {
  crate::{
    delay_line::DelayLine,
    notes::{ADSRStage, Note},
    params::DerivedParams,
  },
  grain_trigger::GrainTrigger,
  grains::Grains,
  linear_adsr::ADSR,
};

pub struct Voices {
  grains: Vec<Grains>,
  adsr: Vec<ADSR>,
  grain_trigger: GrainTrigger,
  fade_time: f32,
  sample_rate: f32,
}

impl Voices {
  pub fn new(sample_rate: f32, fade_time: f32) -> Self {
    Self {
      grains: vec![Grains::new(sample_rate); 8],
      adsr: vec![ADSR::new(sample_rate, 5.); 8],
      grain_trigger: GrainTrigger::new(sample_rate),
      fade_time,
      sample_rate,
    }
  }

  pub fn process(
    &mut self,
    delay_line: &DelayLine,
    notes: &mut Vec<Note>,
    time: f32,
    density: f32,
    stereo: f32,
    speed: f32,
    scan: f32,
    spray: f32,
    midi_enabled: bool,
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    reset_playback: bool,
    start_position_phase: f32,
    derived_params: &DerivedParams,
  ) -> (f32, f32) {
    let DerivedParams {
      duration,
      phase_step_size,
      window_factor,
      fade_factor,
      fade_offset,
    } = *derived_params;

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
          let trigger =
            self
              .grain_trigger
              .process(duration, density, adsr.get_trigger() || reset_playback);

          let grains_out = grains.process(
            delay_line,
            trigger,
            scan,
            spray,
            stereo,
            time,
            start_position_phase,
            phase_step_size,
            speed * adsr.get_speed(),
            window_factor,
            fade_factor,
            fade_offset,
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
        .process(duration, density, reset_playback);
      self.grains[0].process(
        delay_line,
        trigger,
        scan,
        spray,
        stereo,
        time,
        start_position_phase,
        phase_step_size,
        speed,
        window_factor,
        fade_factor,
        fade_offset,
      )
    }
  }
}
