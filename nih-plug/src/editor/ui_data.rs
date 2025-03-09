use crate::grain_stretch_parameters::GrainStretchParameters;
use grain_stretch::WavProcessor;
use nih_plug::{
  nih_log,
  params::Param,
  prelude::{GuiContext, ParamPtr},
};
use nih_plug_vizia::vizia::prelude::*;
use rfd::FileDialog;
use std::sync::Arc;

pub enum ParamChangeEvent {
  SetParam(ParamPtr, f32),
  PickFile,
  SetTimeToFileDuration,
}

#[derive(Lens)]
pub struct UiData {
  pub params: Arc<GrainStretchParameters>,
  pub gui_context: Arc<dyn GuiContext>,
  pub wav_processor: WavProcessor,
}

impl Model for UiData {
  fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
    event.map(|app_event, _| match app_event {
      ParamChangeEvent::SetParam(param_ptr, value) => {
        unsafe {
          self.gui_context.raw_begin_set_parameter(*param_ptr);
          self
            .gui_context
            .raw_set_parameter_normalized(*param_ptr, *value);
          self.gui_context.raw_end_set_parameter(*param_ptr);
        };
      }
      ParamChangeEvent::PickFile => {
        let file_path_param = self.params.file_path.clone();

        cx.spawn(move |cx_proxy| {
          if let Some(file) = FileDialog::new().add_filter("wav", &["wav"]).pick_file() {
            if let Some(file_path) = file.to_str() {
              *file_path_param.lock().unwrap() = file_path.to_string();
              cx_proxy.emit(ParamChangeEvent::SetTimeToFileDuration).ok();
            }
          }
        });
      }
      ParamChangeEvent::SetTimeToFileDuration => {
        let file_path = self.params.file_path.lock().unwrap();

        if file_path.is_empty() {
          return;
        }

        match self.wav_processor.get_duration(file_path.to_string()) {
          Ok(duration) => {
            cx.emit(ParamChangeEvent::SetParam(
              self.params.time.as_ptr(),
              self.params.time.preview_normalized(duration),
            ));
          }
          Err(err) => nih_log!("Failed to load WAV file: {:?}", err),
        };
      }
    });
  }
}
