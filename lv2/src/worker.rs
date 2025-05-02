use {
  crate::DmTimeWarp,
  lv2::prelude::*,
  std::string::String,
  time_warp::{WavFileData, WavProcessor},
};

pub struct WorkData {
  file_path: String,
  sample_rate: f32,
}

impl WorkData {
  pub fn new(file_path: &str, sample_rate: f32) -> Self {
    Self {
      file_path: file_path.to_string(),
      sample_rate,
    }
  }
}

impl Worker for DmTimeWarp {
  type WorkData = WorkData;
  type ResponseData = WavFileData;

  fn work(
    response_handler: &ResponseHandler<Self>,
    data: Self::WorkData,
  ) -> Result<(), WorkerError> {
    if data.file_path.is_empty() {
      return Err(WorkerError::Unknown);
    }
    let wav_file_data = WavProcessor::new(data.sample_rate)
      .read_wav(&data.file_path)
      .or(Err(WorkerError::Unknown))?;
    response_handler
      .respond(wav_file_data)
      .or(Err(WorkerError::Unknown))
  }

  fn work_response(
    &mut self,
    data: Self::ResponseData,
    _features: &mut Self::AudioFeatures,
  ) -> Result<(), WorkerError> {
    self.time_warp.get_delay_line().set_values(&data.samples);
    self.params.set_file_duration(data.duration);
    self.params.reset_playback = true;
    self.worker_is_finished = true;

    Ok(())
  }

  fn end_run(&mut self, _features: &mut Self::AudioFeatures) -> Result<(), WorkerError> {
    Ok(())
  }
}
