use crate::grain_stretch_parameters::GrainStretchParameters;
use nih_plug::prelude::{GuiContext, ParamPtr};
use nih_plug_vizia::vizia::prelude::*;
use rfd::FileDialog;
use std::sync::Arc;

pub enum ParamChangeEvent {
  SetParam(ParamPtr, f32),
  PickFile,
}

#[derive(Lens)]
pub struct UiData {
  pub params: Arc<GrainStretchParameters>,
  pub gui_context: Arc<dyn GuiContext>,
}

impl Model for UiData {
  fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
    event.map(|app_event, _| match app_event {
      ParamChangeEvent::SetParam(param_ptr, value) => {
        unsafe {
          self
            .gui_context
            .raw_set_parameter_normalized(*param_ptr, *value)
        };
      }
      ParamChangeEvent::PickFile => {
        let param = self.params.file_path.clone();

        cx.spawn(move |_cx_proxy| {
          if let Some(file) = FileDialog::new().add_filter("wav", &["wav"]).pick_file() {
            if let Some(file_path) = file.to_str() {
              *param.lock().unwrap() = file_path.to_string();
            }
          }
        });
      }
    });
  }
}
