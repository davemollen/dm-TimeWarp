mod custom_formatters;
use crate::editor;
use custom_formatters::{s2v_f32_ms_then_s, v2s_f32_ms_then_s};
use nih_plug::{
  formatters::{
    s2v_f32_hz_then_khz, s2v_f32_percentage, v2s_f32_hz_then_khz, v2s_f32_percentage,
    v2s_f32_rounded,
  },
  params::{BoolParam, EnumParam, IntParam},
  prelude::{Enum, FloatParam, FloatRange, IntRange, Params},
};
use nih_plug_vizia::ViziaState;
use std::sync::{Arc, Mutex};
use time_warp::MIN_DELAY_TIME;

#[derive(Enum, PartialEq)]
pub enum RecordMode {
  Delay,
  Looper,
}

#[derive(Params)]
pub struct TimeWarpParameters {
  #[persist = "editor-state"]
  pub editor_state: Arc<ViziaState>,

  #[id = "scan"]
  pub scan: FloatParam,

  #[id = "spray"]
  pub spray: FloatParam,

  #[id = "size"]
  pub size: FloatParam,

  #[id = "pitch"]
  pub pitch: FloatParam,

  #[id = "density"]
  pub density: FloatParam,

  #[id = "stretch"]
  pub stretch: FloatParam,

  #[id = "stereo"]
  pub stereo: FloatParam,

  #[id = "record"]
  pub record: BoolParam,

  #[id = "play"]
  pub play: BoolParam,

  #[id = "record_mode"]
  pub record_mode: EnumParam<RecordMode>,

  #[id = "time"]
  pub time: FloatParam,

  #[id = "length"]
  pub length: FloatParam,

  #[id = "highpass"]
  pub highpass: FloatParam,

  #[id = "lowpass"]
  pub lowpass: FloatParam,

  #[id = "recycle"]
  pub recycle: FloatParam,

  #[id = "feedback"]
  pub feedback: FloatParam,

  #[id = "midi_enabled"]
  pub midi_enabled: BoolParam,

  #[id = "voices"]
  pub voices: IntParam,

  #[id = "attack"]
  pub attack: FloatParam,

  #[id = "decay"]
  pub decay: FloatParam,

  #[id = "sustain"]
  pub sustain: FloatParam,

  #[id = "release"]
  pub release: FloatParam,

  #[id = "dry"]
  pub dry: FloatParam,

  #[id = "wet"]
  pub wet: FloatParam,

  #[id = "erase"]
  pub erase: BoolParam,

  #[persist = "file_path"]
  pub file_path: Arc<Mutex<String>>,
}

impl Default for TimeWarpParameters {
  fn default() -> Self {
    Self {
      editor_state: editor::default_state(),

      scan: FloatParam::new("Scan", 0., FloatRange::Linear { min: 0., max: 1. })
        .with_unit(" %")
        .with_value_to_string(v2s_f32_percentage(2))
        .with_string_to_value(s2v_f32_percentage()),

      spray: FloatParam::new(
        "Spray",
        0.,
        FloatRange::Skewed {
          min: 0.,
          max: 500.,
          factor: 0.5,
        },
      )
      .with_value_to_string(v2s_f32_ms_then_s())
      .with_string_to_value(s2v_f32_ms_then_s()),

      size: FloatParam::new("Size", 1., FloatRange::Linear { min: 0., max: 1. })
        .with_unit(" %")
        .with_value_to_string(v2s_f32_percentage(2))
        .with_string_to_value(s2v_f32_percentage()),

      pitch: FloatParam::new(
        "Pitch",
        0.,
        FloatRange::Linear {
          min: -24.,
          max: 24.,
        },
      )
      .with_unit(" st")
      .with_value_to_string(v2s_f32_rounded(2)),

      density: FloatParam::new("Density", 0., FloatRange::Linear { min: 0., max: 1. })
        .with_unit(" %")
        .with_value_to_string(v2s_f32_percentage(2))
        .with_string_to_value(s2v_f32_percentage()),

      stretch: FloatParam::new("Stretch", 1., FloatRange::Linear { min: -2., max: 2. })
        .with_unit(" %")
        .with_value_to_string(v2s_f32_percentage(2))
        .with_string_to_value(s2v_f32_percentage()),

      stereo: FloatParam::new("Stereo", 0., FloatRange::Linear { min: 0., max: 1. })
        .with_unit(" %")
        .with_value_to_string(v2s_f32_percentage(2))
        .with_string_to_value(s2v_f32_percentage()),

      record: BoolParam::new("● / Dub", true),

      play: BoolParam::new("▶ / ◼", true),

      record_mode: EnumParam::new("Record Mode", RecordMode::Delay),

      time: FloatParam::new(
        "Time",
        2000.,
        FloatRange::Skewed {
          min: MIN_DELAY_TIME,
          max: 10000.,
          factor: 0.3,
        },
      )
      .with_value_to_string(v2s_f32_ms_then_s())
      .with_string_to_value(s2v_f32_ms_then_s()),

      length: FloatParam::new("Length", 1., FloatRange::Linear { min: 0., max: 1. })
        .with_unit(" %")
        .with_value_to_string(v2s_f32_percentage(2))
        .with_string_to_value(s2v_f32_percentage()),

      highpass: FloatParam::new(
        "Highpass",
        20.,
        FloatRange::Skewed {
          min: 20.,
          max: 20000.,
          factor: 0.2,
        },
      )
      .with_value_to_string(v2s_f32_hz_then_khz(2))
      .with_string_to_value(s2v_f32_hz_then_khz()),

      lowpass: FloatParam::new(
        "Lowpass",
        20000.,
        FloatRange::Skewed {
          min: 20.,
          max: 20000.,
          factor: 0.2,
        },
      )
      .with_value_to_string(v2s_f32_hz_then_khz(2))
      .with_string_to_value(s2v_f32_hz_then_khz()),

      recycle: FloatParam::new("Recycle", 0., FloatRange::Linear { min: 0., max: 1. })
        .with_unit(" %")
        .with_value_to_string(v2s_f32_percentage(2))
        .with_string_to_value(s2v_f32_percentage()),

      feedback: FloatParam::new("Feedback", 1., FloatRange::Linear { min: 0., max: 1. })
        .with_unit(" %")
        .with_value_to_string(v2s_f32_percentage(2))
        .with_string_to_value(s2v_f32_percentage()),

      midi_enabled: BoolParam::new("MIDI", false),

      voices: IntParam::new("Voices", 1, IntRange::Linear { min: 1, max: 8 }),

      attack: FloatParam::new(
        "Attack",
        10.,
        FloatRange::Skewed {
          min: 0.1,
          max: 5000.,
          factor: 0.2,
        },
      )
      .with_value_to_string(v2s_f32_ms_then_s())
      .with_string_to_value(s2v_f32_ms_then_s()),

      decay: FloatParam::new(
        "Decay",
        300.,
        FloatRange::Skewed {
          min: 1.,
          max: 15000.,
          factor: 0.2,
        },
      )
      .with_value_to_string(v2s_f32_ms_then_s())
      .with_string_to_value(s2v_f32_ms_then_s()),

      sustain: FloatParam::new("Sustain", 1., FloatRange::Linear { min: 0., max: 1. })
        .with_value_to_string(v2s_f32_rounded(2)),

      release: FloatParam::new(
        "Release",
        2000.,
        FloatRange::Skewed {
          min: 1.,
          max: 15000.,
          factor: 0.3,
        },
      )
      .with_value_to_string(v2s_f32_ms_then_s())
      .with_string_to_value(s2v_f32_ms_then_s()),

      dry: FloatParam::new(
        "Dry",
        0.,
        FloatRange::Skewed {
          min: -70.,
          max: 12.,
          factor: 2.,
        },
      )
      .with_unit(" dB")
      .with_value_to_string(Arc::new(move |value| {
        if value == -70. {
          "-inf".to_string()
        } else {
          format!("{:.2}", value)
        }
      })),

      wet: FloatParam::new(
        "Wet",
        0.,
        FloatRange::Skewed {
          min: -70.,
          max: 12.,
          factor: 2.,
        },
      )
      .with_unit(" dB")
      .with_value_to_string(Arc::new(move |value| {
        if value == -70. {
          "-inf".to_string()
        } else {
          format!("{:.2}", value)
        }
      })),

      erase: BoolParam::new("Erase", false),

      file_path: Arc::new(Mutex::new("".to_string())),
    }
  }
}
