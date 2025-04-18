mod smooth;
mod stopwatch;
mod wav_processor;
pub use smooth::Smoother;
use {
  crate::shared::float_ext::FloatExt,
  crate::stereo_delay_line::StereoDelayLine,
  smooth::{LinearSmooth, LogarithmicSmooth},
  std::sync::{Arc, Mutex},
  stopwatch::Stopwatch,
  wav_processor::WavProcessor,
};

#[derive(PartialEq)]
pub enum TimeMode {
  Delay,
  Looper,
}

pub struct Params {
  pub scan: f32,
  pub spray: f32,
  pub size: f32,
  pub speed: f32,
  pub density: f32,
  pub stretch: f32,
  pub recording_gain: LinearSmooth,
  pub playback_gain: LinearSmooth,
  pub time: LogarithmicSmooth,
  pub highpass: LinearSmooth,
  pub lowpass: LinearSmooth,
  pub feedback: LinearSmooth,
  pub recycle: LinearSmooth,
  pub dry: LinearSmooth,
  pub wet: LinearSmooth,
  pub midi_enabled: bool,
  is_initialized: bool,
  pub attack: LinearSmooth,
  pub decay: LinearSmooth,
  pub sustain: LinearSmooth,
  pub release: LinearSmooth,
  pub reset_playback: bool,
  wav_processor: WavProcessor,
  loaded_file_path: Option<String>,
  file_duration: Option<f32>,
  prev_file_duration: Option<f32>,
  loop_duration: Option<f32>,
  stopwatch: Stopwatch,
  play: bool,
  clear: bool,
}

impl Params {
  pub fn new(sample_rate: f32) -> Self {
    Self {
      scan: 0.,
      spray: 0.,
      size: 0.,
      speed: 0.,
      density: 0.,
      stretch: 0.,
      recording_gain: LinearSmooth::new(sample_rate, 20.),
      playback_gain: LinearSmooth::new(sample_rate, 20.),
      time: LogarithmicSmooth::new(sample_rate, 0.3),
      highpass: LinearSmooth::new(sample_rate, 20.),
      lowpass: LinearSmooth::new(sample_rate, 20.),
      feedback: LinearSmooth::new(sample_rate, 20.),
      recycle: LinearSmooth::new(sample_rate, 20.),
      dry: LinearSmooth::new(sample_rate, 20.),
      wet: LinearSmooth::new(sample_rate, 20.),
      midi_enabled: false,
      is_initialized: false,
      attack: LinearSmooth::new(sample_rate, 20.),
      decay: LinearSmooth::new(sample_rate, 20.),
      sustain: LinearSmooth::new(sample_rate, 20.),
      release: LinearSmooth::new(sample_rate, 20.),
      reset_playback: false,
      wav_processor: WavProcessor::new(sample_rate),
      loaded_file_path: None,
      file_duration: None,
      prev_file_duration: None,
      loop_duration: None,
      stopwatch: Stopwatch::new(sample_rate),
      play: true,
      clear: false,
    }
  }

  pub fn set(
    &mut self,
    scan: f32,
    spray: f32,
    size: f32,
    speed: f32,
    density: f32,
    stretch: f32,
    record: bool,
    play: bool,
    time_mode: TimeMode,
    time: f32,
    time_multiply: f32,
    highpass: f32,
    lowpass: f32,
    feedback: f32,
    recycle: f32,
    dry: f32,
    wet: f32,
    midi_enabled: bool,
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    file_path: Arc<Mutex<String>>,
    clear: bool,
    delay_line: &mut StereoDelayLine,
  ) {
    self.reset_playback = false;
    self.scan = scan;
    self.spray = spray;
    self.size = size.powf(0.333);
    self.speed = speed;
    self.density = density * density;
    self.stretch = stretch;
    self.midi_enabled = midi_enabled;

    let recording_gain = self.get_recording_gain(record, play, &time_mode);
    let playback_gain = self.get_playback_gain(play, &time_mode);
    let sustain = sustain.dbtoa();
    let dry = dry.dbtoa();
    let wet = wet.dbtoa();

    self.load_file(file_path.lock().unwrap().as_str(), delay_line);

    if clear {
      self.loaded_file_path = None;
      *file_path.lock().unwrap() = "".to_string();
      self.file_duration = None;
      self.stopwatch.reset();
      self.loop_duration = None;
      delay_line.reset();
      self.clear = false;
    }

    if self.is_initialized {
      self.recording_gain.set_target(recording_gain);
      self.playback_gain.set_target(playback_gain);
      self.set_time(time_mode, record, time, time_multiply);
      self.highpass.set_target(highpass);
      self.lowpass.set_target(lowpass);
      self.feedback.set_target(feedback);
      self.recycle.set_target(recycle);
      self.dry.set_target(dry);
      self.wet.set_target(wet);
      self.attack.set_target(attack);
      self.decay.set_target(decay);
      self.sustain.set_target(sustain);
      self.release.set_target(release);
    } else {
      self.recording_gain.reset(recording_gain);
      self.playback_gain.reset(playback_gain);
      self.reset_time(time, time_multiply);
      self.highpass.reset(highpass);
      self.lowpass.reset(lowpass);
      self.feedback.reset(feedback);
      self.recycle.reset(recycle);
      self.dry.reset(dry);
      self.wet.reset(wet);
      self.attack.reset(attack);
      self.decay.reset(decay);
      self.sustain.reset(sustain);
      self.release.reset(release);
      self.is_initialized = true;
    }
    self.play = play;
    self.clear = clear;
  }

  fn get_recording_gain(&mut self, record: bool, play: bool, time_mode: &TimeMode) -> f32 {
    match (record, time_mode, self.loop_duration) {
      (true, TimeMode::Looper, None) => {
        let start_playback = play && !self.play;
        if start_playback {
          0.
        } else {
          1.
        }
      }
      (false, _, _) => 0.,
      (true, _, _) => 1.,
    }
  }

  fn get_playback_gain(&mut self, play: bool, time_mode: &TimeMode) -> f32 {
    match (play, time_mode, self.loop_duration) {
      (true, TimeMode::Looper, None) => 0.,
      (true, _, _) => {
        self.reset_playback = !self.play;
        1.
      }
      (false, _, _) => 0.,
    }
  }

  fn load_file(&mut self, file_path: &str, delay_line: &mut StereoDelayLine) {
    if file_path.is_empty()
      || self
        .loaded_file_path
        .as_ref()
        .is_some_and(|x| *x == file_path)
    {
      return;
    }
    if let Ok(samples) = self.wav_processor.read_wav(file_path) {
      delay_line.set_values(&samples);
    };
    if let Ok(duration) = self.wav_processor.get_duration(file_path) {
      self.file_duration = Some(duration);
    };
    self.loaded_file_path = Some(file_path.to_string());
    self.reset_playback = true;
  }

  fn set_time(&mut self, time_mode: TimeMode, record: bool, time: f32, time_multiply: f32) {
    match (
      self.file_duration,
      self.prev_file_duration,
      self.loop_duration,
      time_mode,
    ) {
      (Some(file_duration), Some(prev_file_duration), _, _) => {
        if file_duration == prev_file_duration {
          self.time.set_target(file_duration * time_multiply);
        } else {
          self.time.reset(file_duration * time_multiply);
        }
      }
      (Some(file_duration), None, _, _) => {
        self.time.reset(file_duration * time_multiply);
      }
      (_, _, None, TimeMode::Looper) => {
        if let Some(loop_duration) = self.stopwatch.process(record) {
          self.time.reset(loop_duration * time_multiply);
          self.loop_duration = Some(loop_duration);
        }
      }
      (_, _, Some(loop_duration), TimeMode::Looper) => {
        self.time.set_target(loop_duration * time_multiply);
      }
      (_, _, _, _) => {
        self.time.set_target(time);
      }
    }
  }

  fn reset_time(&mut self, time: f32, time_multiply: f32) {
    match (self.file_duration, self.loop_duration) {
      (Some(dur), _) => self.time.reset(dur * time_multiply),
      (None, Some(dur)) => self.time.reset(dur * time_multiply),
      (None, None) => {
        self.time.reset(time);
      }
    }
  }
}
