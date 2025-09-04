mod custom_formatters;
use {
  crate::{
    editor,
    time_warp_parameters::custom_formatters::{
      s2v_f32_ms_then_s, s2v_f32_synced_time, s2v_size, v2s_f32_ms_then_s, v2s_f32_synced_time,
      v2s_size,
    },
  },
  nih_plug::{
    formatters::{
      s2v_f32_hz_then_khz, s2v_f32_percentage, v2s_f32_hz_then_khz, v2s_f32_percentage,
      v2s_f32_rounded,
    },
    params::{BoolParam, EnumParam, IntParam},
    prelude::{AtomicF32, Enum, FloatParam, FloatRange, IntRange, Params},
  },
  std::sync::{Arc, Mutex},
  time_warp::{MAX_DENSITY, MIN_DELAY_TIME, MIN_DENSITY},
  vizia_plug::ViziaState,
};

const MAX_PARAM_DELAY_TIME: f32 = 10000.;

#[derive(Enum, PartialEq)]
pub enum SampleMode {
  Delay,
  Looper,
  Sampler,
}

#[derive(Params)]
pub struct TimeWarpParameters {
  #[persist = "editor-state"]
  pub editor_state: Arc<ViziaState>,

  #[id = "scan"]
  pub scan: FloatParam,

  #[id = "spray"]
  pub spray: FloatParam,

  #[id = "freeze"]
  pub freeze: BoolParam,

  #[id = "stretch"]
  pub stretch: FloatParam,

  #[id = "size"]
  pub size: FloatParam,

  #[id = "density"]
  pub density: FloatParam,

  #[id = "stereo"]
  pub stereo: FloatParam,

  #[id = "detune"]
  pub detune: IntParam,

  #[id = "pitch"]
  pub pitch: IntParam,

  #[id = "record"]
  pub record: BoolParam,

  #[id = "play"]
  pub play: BoolParam,

  #[id = "sample_mode"]
  pub sample_mode: EnumParam<SampleMode>,

  #[id = "sync"]
  pub sync: BoolParam,

  #[id = "time"]
  pub time: FloatParam,

  #[id = "division"]
  pub division: IntParam,

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

  #[id = "sync_position"]
  pub sync_position: BoolParam,

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

  #[persist = "max_size"]
  pub max_size: Arc<AtomicF32>,
}

impl Default for TimeWarpParameters {
  fn default() -> Self {
    let max_size = Arc::new(AtomicF32::new(MAX_PARAM_DELAY_TIME));

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

      freeze: BoolParam::new("Freeze", false),

      stretch: FloatParam::new("Stretch", 1., FloatRange::Linear { min: -2., max: 2. })
        .with_unit(" %")
        .with_value_to_string(v2s_f32_percentage(2))
        .with_string_to_value(s2v_f32_percentage()),

      size: FloatParam::new("Size", 1., FloatRange::Linear { min: 0., max: 1. })
        .with_value_to_string(v2s_size(max_size.clone()))
        .with_string_to_value(s2v_size(max_size.clone())),

      density: FloatParam::new(
        "Density",
        MIN_DENSITY,
        FloatRange::Skewed {
          min: MIN_DENSITY,
          max: MAX_DENSITY,
          factor: 0.5,
        },
      )
      .with_value_to_string(v2s_f32_rounded(2)),

      stereo: FloatParam::new("Stereo", 0., FloatRange::Linear { min: 0., max: 1. })
        .with_unit(" %")
        .with_value_to_string(v2s_f32_percentage(2))
        .with_string_to_value(s2v_f32_percentage()),

      detune: IntParam::new(
        "Detune",
        0,
        IntRange::Linear {
          min: -100,
          max: 100,
        },
      )
      .with_unit(" ct"),

      pitch: IntParam::new("Pitch", 0, IntRange::Linear { min: -24, max: 24 }).with_unit(" st"),

      record: BoolParam::new("Record / Dub", false),

      play: BoolParam::new("Play / Stop", true),

      sample_mode: EnumParam::new("Sample Mode", SampleMode::Delay),

      sync: BoolParam::new("Sync", false),

      time: FloatParam::new(
        "Time",
        2000.,
        FloatRange::Skewed {
          min: MIN_DELAY_TIME,
          max: MAX_PARAM_DELAY_TIME,
          factor: 0.3,
        },
      )
      .with_value_to_string(v2s_f32_ms_then_s())
      .with_string_to_value(s2v_f32_ms_then_s()),

      division: IntParam::new("Division", 15, IntRange::Linear { min: 0, max: 20 })
        .with_value_to_string(v2s_f32_synced_time())
        .with_string_to_value(s2v_f32_synced_time()),

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

      sync_position: BoolParam::new("Sync Pos.", false),

      voices: IntParam::new("Voices", 1, IntRange::Linear { min: 1, max: 8 }),

      attack: FloatParam::new(
        "Attack",
        1.,
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
        5.,
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
        5.,
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
          format!("{:.1}", value)
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
          format!("{:.1}", value)
        }
      })),

      erase: BoolParam::new("Erase", false),

      file_path: Arc::new(Mutex::new("".to_string())),

      max_size,
    }
  }
}
