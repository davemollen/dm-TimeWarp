use crossbeam_channel::{Receiver, Sender}; // TODO: check other crates like omange, ringbuf or rtrb as an alternative
use nih_plug::prelude::AtomicF32;
use std::sync::{atomic::Ordering, Arc, Mutex};
use time_warp::{WavFileData, WavProcessor};

pub enum Task {
  LoadFile(String, bool),
}

#[derive(Clone)]
pub struct FileLoader {
  sample_rate: Arc<AtomicF32>,
  file_path_param: Arc<Mutex<String>>,
  sender: Sender<WavFileData>,
  receiver: Receiver<WavFileData>,
}

impl FileLoader {
  pub fn new(sample_rate: f32, file_path_param: Arc<Mutex<String>>) -> Self {
    let (sender, receiver) = crossbeam_channel::bounded(16);
    Self {
      sample_rate: Arc::new(AtomicF32::new(sample_rate)),
      file_path_param,
      sender,
      receiver,
    }
  }

  pub fn handle_task(&self, task: Task) {
    match task {
      Task::LoadFile(file_path, should_update_file_path) => {
        if file_path.is_empty() {
          return;
        }
        let wav_file_data =
          match WavProcessor::new(self.sample_rate.load(Ordering::Relaxed)).read_wav(&file_path) {
            Ok(data) => data,
            Err(_) => {
              return;
            }
          };
        match self.sender.try_send(wav_file_data) {
          Ok(_) => {
            if should_update_file_path {
              *self.file_path_param.lock().unwrap() = file_path.clone();
            }
          }
          _ => {}
        }
      }
    }
  }

  pub fn try_receive_data(&self) -> Option<WavFileData> {
    self.receiver.try_recv().ok()
  }

  pub fn set_sample_rate(&mut self, sample_rate: f32) {
    self.sample_rate.store(sample_rate, Ordering::Relaxed);
  }
}
