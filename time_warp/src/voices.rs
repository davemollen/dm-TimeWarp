mod grain_trigger;
mod grains;
mod linear_adsr;
mod start_position_phasor;
use {
  crate::{
    delay_line::DelayLine,
    notes::{ADSRStage, Note},
    shared::float_ext::FloatExt,
    FADE_TIME, MAX_DENSITY, MIN_DELAY_TIME, MIN_DENSITY,
  },
  grain_trigger::GrainTrigger,
  grains::Grains,
  linear_adsr::ADSR,
  start_position_phasor::StartPositionPhasor,
};

pub struct Voices {
  grains: Vec<Grains>,
  adsr: Vec<ADSR>,
  phasors: Vec<StartPositionPhasor>,
  grain_trigger: GrainTrigger,
  sample_rate: f32,
}

impl Voices {
  pub fn new(sample_rate: f32) -> Self {
    Self {
      grains: vec![Grains::new(sample_rate); 8],
      adsr: vec![ADSR::new(sample_rate, 5.); 8],
      phasors: vec![StartPositionPhasor::new(sample_rate); 8],
      grain_trigger: GrainTrigger::new(sample_rate),
      sample_rate,
    }
  }

  pub fn process(
    &mut self,
    delay_line: &DelayLine,
    notes: &mut Vec<Note>,
    size: f32,
    time: f32,
    density: f32,
    stereo: f32,
    speed: f32,
    stretch: f32,
    scan: f32,
    spray: f32,
    midi_enabled: bool,
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    reset_playback: bool,
    phase_offset: f32,
  ) -> (f32, f32) {
    let duration = size * (time - MIN_DELAY_TIME) + MIN_DELAY_TIME; // range from min delay time to time
    let normalized_density = (density - MIN_DENSITY) / (MAX_DENSITY - MIN_DENSITY);
    let grain_duration = duration + FADE_TIME * (1. - normalized_density);
    let phase_step_size = grain_duration.mstosamps(self.sample_rate).recip();
    let min_window_factor = 2.;
    let max_window_factor = grain_duration / FADE_TIME;
    let window_factor = max_window_factor.mix(min_window_factor, normalized_density);
    let is_in_granular_mode = size < 1. || density > 1.;
    let freq = 1000. / time;

    if midi_enabled {
      notes
        .iter_mut()
        .filter(|n| *n.get_adsr_stage() != ADSRStage::Idle)
        .zip(self.grains.iter_mut())
        .zip(self.adsr.iter_mut())
        .zip(self.phasors.iter_mut())
        .fold((0., 0.), |result, (((note, grains), adsr), phasor)| {
          let speed = speed * adsr.get_speed();
          let gain = adsr.process(note, attack, decay, sustain, release);
          let reset = adsr.get_trigger() || reset_playback;
          if reset {
            phasor.reset(phase_offset);
            grains.reset();
          }
          let start_position_phase = phasor.process(freq, speed, stretch, is_in_granular_mode);
          let trigger = self.grain_trigger.process(duration, density, reset);
          let grains_out = grains.process(
            delay_line,
            trigger,
            scan,
            spray,
            stereo,
            time,
            start_position_phase,
            phase_step_size,
            speed,
            stretch < 0.,
            window_factor,
          );
          (
            result.0 + grains_out.0 * gain,
            result.1 + grains_out.1 * gain,
          )
        })
    } else {
      if reset_playback {
        self.phasors[0].reset(phase_offset);
        self.grains[0].reset();
      }
      let start_position_phase = self.phasors[0].process(freq, speed, stretch, is_in_granular_mode);
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
        stretch < 0.,
        window_factor,
      )
    }
  }
}
