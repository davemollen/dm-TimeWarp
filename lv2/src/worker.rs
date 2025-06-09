use {
  crate::DmTimeWarp,
  lv2::prelude::*,
  std::string::String,
  time_warp::{WavFileData, WavProcessor},
};

pub enum WorkRequest {
  LoadFile(String, f32, usize),
  FlushBuffer(usize),
}

pub enum WorkResponseData {
  LoadFile(WavFileData),
  FlushBuffer(Vec<(f32, f32)>),
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
        let mut wav_file_data = WavProcessor::new(sample_rate)
          .read_wav(&file_path)
          .or(Err(WorkerError::Unknown))?;
        wav_file_data.samples.resize(size, (0., 0.));

        response_handler
          .respond(WorkResponseData::LoadFile(wav_file_data))
          .or(Err(WorkerError::Unknown))
      }
      WorkRequest::FlushBuffer(size) => response_handler
        .respond(WorkResponseData::FlushBuffer(vec![(0., 0.); size]))
        .or(Err(WorkerError::Unknown)),
    }
  }

  fn work_response(
    &mut self,
    data: Self::ResponseData,
    _features: &mut Self::AudioFeatures,
  ) -> Result<(), WorkerError> {
    match data {
      WorkResponseData::LoadFile(WavFileData {
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

  fn end_run(&mut self, _features: &mut Self::AudioFeatures) -> Result<(), WorkerError> {
    Ok(())
  }
}
