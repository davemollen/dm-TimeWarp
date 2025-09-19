mod grain_trigger;
mod grains;
mod linear_adsr;
mod start_position_phasor;
use {
  crate::{
    delay_line::DelayLine,
    notes::{ADSRStage, Note},
    shared::float_ext::FloatExt,
    CENTER_GRAIN_DURATION, FADE_TIME, MAX_DENSITY, MIN_DELAY_TIME, MIN_DENSITY,
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
  grain_triggers: Vec<GrainTrigger>,
  sample_rate: f32,
  has_active_notes: bool,
}

impl Voices {
  pub fn new(sample_rate: f32) -> Self {
    Self {
      grains: vec![Grains::new(sample_rate); 8],
      adsr: vec![ADSR::new(sample_rate, 5.); 8],
      phasors: vec![StartPositionPhasor::new(sample_rate); 8],
      grain_triggers: vec![GrainTrigger::new(sample_rate); 8],
      sample_rate,
      has_active_notes: false,
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
    sync_position: bool,
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    reset_playback: bool,
    phase_offset: f32,
  ) -> (f32, f32) {
    let grain_duration = Self::map_size_to_grain_duration(size, time);
    let normalized_density = (density - MIN_DENSITY) / (MAX_DENSITY - MIN_DENSITY);
    let extended_grain_duration = grain_duration + FADE_TIME * (1. - normalized_density);
    let phase_step_size = extended_grain_duration.mstosamps(self.sample_rate).recip();
    let min_window_factor = 2.;
    let max_window_factor = extended_grain_duration / FADE_TIME;
    let window_factor = max_window_factor.mix(min_window_factor, normalized_density);
    let fade_factor = time / FADE_TIME;
    let fade_offset = fade_factor.recip() + 1.;
    let is_in_granular_mode = size < 1. || density > 1.;
    let freq = 1000. / time;

    if midi_enabled {
      if sync_position {
        let has_active_notes = notes
          .iter()
          .any(|note| *note.get_adsr_stage() != ADSRStage::Idle);
        let reset = (has_active_notes && !self.has_active_notes) || reset_playback;
        self.has_active_notes = has_active_notes;
        if reset {
          self.phasors[0].reset(phase_offset);
          self.grains.iter_mut().for_each(|grain| grain.reset());
        }
        let start_position_phase =
          self.phasors[0].process(freq, speed, stretch, is_in_granular_mode);
        notes
          .iter_mut()
          .zip(self.grains.iter_mut())
          .zip(self.adsr.iter_mut())
          .zip(self.grain_triggers.iter_mut())
          .fold(
            (0., 0.),
            |result, (((note, grains), adsr), grain_trigger)| {
              if *note.get_adsr_stage() == ADSRStage::Idle {
                return result;
              }
              let speed = speed * adsr.get_speed();
              let gain = adsr.process(note, attack, decay, sustain, release);
              let trigger = grain_trigger.process(grain_duration, density, reset);
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
                fade_factor,
                fade_offset,
              );
              (
                result.0 + grains_out.0 * gain,
                result.1 + grains_out.1 * gain,
              )
            },
          )
      } else {
        notes
          .iter_mut()
          .zip(self.grains.iter_mut())
          .zip(self.adsr.iter_mut())
          .zip(self.phasors.iter_mut())
          .zip(self.grain_triggers.iter_mut())
          .fold(
            (0., 0.),
            |result, ((((note, grains), adsr), phasor), grain_trigger)| {
              if *note.get_adsr_stage() == ADSRStage::Idle {
                return result;
              }
              let speed = speed * adsr.get_speed();
              let gain = adsr.process(note, attack, decay, sustain, release);
              let reset = adsr.get_trigger() || reset_playback;
              if reset {
                phasor.reset(phase_offset);
                grains.reset();
              }
              let start_position_phase = phasor.process(freq, speed, stretch, is_in_granular_mode);
              let trigger = grain_trigger.process(grain_duration, density, reset);
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
                fade_factor,
                fade_offset,
              );
              (
                result.0 + grains_out.0 * gain,
                result.1 + grains_out.1 * gain,
              )
            },
          )
      }
    } else {
      if reset_playback {
        self.phasors[0].reset(phase_offset);
        self.grains[0].reset();
      }
      let start_position_phase = self.phasors[0].process(freq, speed, stretch, is_in_granular_mode);
      let trigger = self.grain_triggers[0].process(grain_duration, density, reset_playback);
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
        fade_factor,
        fade_offset,
      )
    }
  }

  fn map_size_to_grain_duration(size: f32, time: f32) -> f32 {
    if size < 0.5 {
      size * 2. * (CENTER_GRAIN_DURATION - MIN_DELAY_TIME) + MIN_DELAY_TIME
    } else {
      let range = (size - 0.5) * 2.;
      range * range * (time - CENTER_GRAIN_DURATION) + CENTER_GRAIN_DURATION
    }
    .min(time)
  }
}
