use crossbeam_channel::{Receiver, Sender}; // TODO: check other crates like omange, ringbuf or rtrb as an alternative
use nih_plug::prelude::AtomicF32;
use std::sync::{atomic::Ordering, Arc, Mutex};
use time_warp::{AudioFileData, AudioFileProcessor};

pub enum WorkerRequest {
  LoadFile(String, bool),
  FlushBuffer,
}

pub enum WorkerResponseData {
  LoadFile(AudioFileData),
  FlushBuffer(Vec<f32>),
}

#[derive(Clone)]
pub struct Worker {
  sample_rate: Arc<AtomicF32>,
  file_path_param: Arc<Mutex<String>>,
  sender: Sender<WorkerResponseData>,
  receiver: Receiver<WorkerResponseData>,
  delay_line_size: usize,
}

impl Worker {
  pub fn new(
    sample_rate: f32,
    file_path_param: Arc<Mutex<String>>,
    delay_line_size: usize,
  ) -> Self {
    let (sender, receiver) = crossbeam_channel::bounded(1);
    Self {
      sample_rate: Arc::new(AtomicF32::new(sample_rate)),
      file_path_param,
      sender,
      receiver,
      delay_line_size,
    }
  }

  pub fn initialize(&mut self, sample_rate: f32, size: usize) {
    self.sample_rate.store(sample_rate, Ordering::Relaxed);
    self.delay_line_size = size;
  }

  pub fn handle_task(&self, task: WorkerRequest) {
    match task {
      WorkerRequest::LoadFile(file_path, should_update_file_path) => {
        if file_path.is_empty() {
          return;
        }
        let audio_file_data = match AudioFileProcessor::new(
          self.sample_rate.load(Ordering::Relaxed),
          self.delay_line_size,
        )
        .read(&file_path)
        {
          Ok(data) => data,
          Err(_) => {
            return;
          }
        };

        match self
          .sender
          .try_send(WorkerResponseData::LoadFile(audio_file_data))
        {
          Ok(_) => {
            if should_update_file_path {
              // should not replace file_path on initial load
              *self.file_path_param.lock().unwrap() = file_path.clone();
            }
          }
          _ => {}
        }
      }
      WorkerRequest::FlushBuffer => {
        let empty_buffer = vec![0.; self.delay_line_size];
        self
          .sender
          .try_send(WorkerResponseData::FlushBuffer(empty_buffer))
          .ok();
      }
    }
  }

  pub fn try_receive_data(&self) -> Option<WorkerResponseData> {
    self.receiver.try_recv().ok()
  }
}
