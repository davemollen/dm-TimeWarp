use {
  crate::DmTimeWarp,
  lv2::prelude::*,
  std::string::String,
  time_warp::{AudioFileData, AudioFileProcessor},
};

pub enum WorkRequest {
  LoadFile(String, f32, usize),
  FlushBuffer(usize),
}

pub enum WorkResponseData {
  LoadFile(AudioFileData),
  FlushBuffer(Vec<f32>),
}

impl Worker for DmTimeWarp {
  type WorkData = WorkRequest;
  type ResponseData = WorkResponseData;

  fn work(
    response_handler: &ResponseHandler<Self>,
    data: Self::WorkData,
  ) -> Result<(), WorkerError> {
    match data {
      WorkRequest::LoadFile(file_path, sample_rate, size) => {
        if file_path.is_empty() {
          return Err(WorkerError::Unknown);
        }
        let mut audio_file_data = AudioFileProcessor::new(sample_rate)
          .read(&file_path)
          .or(Err(WorkerError::Unknown))?;
        audio_file_data.samples.resize(size, 0.);

        response_handler
          .respond(WorkResponseData::LoadFile(audio_file_data))
          .or(Err(WorkerError::Unknown))
      }
      WorkRequest::FlushBuffer(size) => response_handler
        .respond(WorkResponseData::FlushBuffer(vec![0.; size]))
        .or(Err(WorkerError::Unknown)),
    }
  }

  fn work_response(
    &mut self,
    data: Self::ResponseData,
    _features: &mut Self::AudioFeatures,
  ) -> Result<(), WorkerError> {
    match data {
      WorkResponseData::LoadFile(AudioFileData {
        samples,
        duration_in_samples,
        duration_in_ms,
      }) => {
        self
          .time_warp
          .set_delay_line_values(samples, duration_in_samples);
        self.params.set_file_duration(duration_in_ms);
        self.params.set_reset_playback(true);
        self.worker_is_finished = true;
      }
      WorkResponseData::FlushBuffer(samples) => {
        self.time_warp.set_delay_line_values(samples, 0);
      }
    }

    Ok(())
  }
}
