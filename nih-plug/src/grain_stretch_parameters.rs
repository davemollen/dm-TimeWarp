use grain_stretch::{MAX_DELAY_TIME, MIN_DELAY_TIME};
use nih_plug::{
  formatters::{
    s2v_f32_hz_then_khz, s2v_f32_percentage, v2s_f32_hz_then_khz, v2s_f32_percentage,
    v2s_f32_rounded,
  },
  params::BoolParam,
  prelude::{FloatParam, FloatRange, Params},
};
use nih_plug_vizia::ViziaState;
use std::sync::Arc;
mod custom_formatters;
use crate::editor;
use custom_formatters::{s2v_f32_ms_then_s, v2s_f32_ms_then_s};

#[derive(Params)]
pub struct GrainStretchParameters {
  #[persist = "editor-state"]
  pub editor_state: Arc<ViziaState>,

  #[id = "scan"]
  pub scan: FloatParam,

  #[id = "spray"]
  pub spray: FloatParam,

  #[id = "size"]
  pub size: FloatParam,

  #[id = "speed"]
  pub speed: FloatParam,

  #[id = "density"]
  pub density: FloatParam,

  #[id = "stretch"]
  pub stretch: FloatParam,

  #[id = "record"]
  pub record: BoolParam,

  #[id = "time"]
  pub time: FloatParam,

  #[id = "highpass"]
  pub highpass: FloatParam,

  #[id = "lowpass"]
  pub lowpass: FloatParam,

  #[id = "overdub"]
  pub overdub: FloatParam,

  #[id = "recycle"]
  pub recycle: FloatParam,

  #[id = "dry"]
  pub dry: FloatParam,

  #[id = "wet"]
  pub wet: FloatParam,
}

impl Default for GrainStretchParameters {
  fn default() -> Self {
    Self {
      editor_state: editor::default_state(),

      scan: FloatParam::new("Scan", 0., FloatRange::Linear { min: 0., max: 1. })
        .with_unit(" %")
        .with_value_to_string(v2s_f32_percentage(2))
        .with_string_to_value(s2v_f32_percentage()),

      spray: FloatParam::new("Spray", 0., FloatRange::Linear { min: 0., max: 1. })
        .with_unit(" %")
        .with_value_to_string(v2s_f32_percentage(2))
        .with_string_to_value(s2v_f32_percentage()),

      size: FloatParam::new("Size", 0., FloatRange::Linear { min: 0., max: 1. })
        .with_unit(" %")
        .with_value_to_string(v2s_f32_percentage(2))
        .with_string_to_value(s2v_f32_percentage()),

      speed: FloatParam::new("Speed", 1., FloatRange::Linear { min: -4., max: 4. })
        .with_value_to_string(v2s_f32_rounded(2)),

      density: FloatParam::new("Density", 0., FloatRange::Linear { min: 0., max: 1. })
        .with_unit(" %")
        .with_value_to_string(v2s_f32_percentage(2))
        .with_string_to_value(s2v_f32_percentage()),

      stretch: FloatParam::new("Stretch", 1., FloatRange::Linear { min: 0., max: 2. })
        .with_unit(" %")
        .with_value_to_string(v2s_f32_percentage(2))
        .with_string_to_value(s2v_f32_percentage()),

      record: BoolParam::new("Record", true),

      time: FloatParam::new(
        "Time",
        2000.,
        FloatRange::Skewed {
          min: MIN_DELAY_TIME,
          max: MAX_DELAY_TIME,
          factor: 0.3,
        },
      )
      .with_value_to_string(v2s_f32_ms_then_s())
      .with_string_to_value(s2v_f32_ms_then_s()),

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

      overdub: FloatParam::new("Overdub", 0., FloatRange::Linear { min: 0., max: 1. })
        .with_unit(" %")
        .with_value_to_string(v2s_f32_percentage(2))
        .with_string_to_value(s2v_f32_percentage()),

      recycle: FloatParam::new("Recycle", 0., FloatRange::Linear { min: 0., max: 1. })
        .with_unit(" %")
        .with_value_to_string(v2s_f32_percentage(2))
        .with_string_to_value(s2v_f32_percentage()),

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
    }
  }
}
