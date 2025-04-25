use crate::DmTimeWarp;
use crate::{time_warp_parameters::TimeWarpParameters, Task};
use nih_plug::prelude::{AsyncExecutor, GuiContext, ParamPtr};
use nih_plug_vizia::vizia::prelude::*;
use rfd::FileDialog;
use std::sync::Arc;

pub enum ParamChangeEvent {
  SetParam(ParamPtr, f32),
  PickFile(AsyncExecutor<DmTimeWarp>),
}

#[derive(Lens)]
pub struct UiData {
  pub params: Arc<TimeWarpParameters>,
  pub gui_context: Arc<dyn GuiContext>,
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
      ParamChangeEvent::PickFile(executor) => {
        let executor = executor.clone();

        cx.spawn(move |_cx_proxy| {
          if let Some(file) = FileDialog::new().add_filter("wav", &["wav"]).pick_file() {
            executor.execute_background(Task::LoadFile(file.to_string_lossy().into_owned(), true));
          }
        });
      }
    });
  }
}
