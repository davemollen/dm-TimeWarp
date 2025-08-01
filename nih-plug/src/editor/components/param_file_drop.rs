use crate::{worker::WorkerRequest, DmTimeWarp};
use nih_plug::prelude::AsyncExecutor;
use rfd::FileDialog;
use std::path::PathBuf;
use vizia_plug::vizia::{icons::ICON_CHEVRON_DOWN, prelude::*};

enum ParamFileDropEvent {
  PickFileFromDialog,
  SetFilePath(PathBuf),
}

pub struct ParamFileDrop {
  async_executor: AsyncExecutor<DmTimeWarp>,
}

impl ParamFileDrop {
  pub fn new<L>(
    cx: &mut Context,
    async_executor: AsyncExecutor<DmTimeWarp>,
    lens: L,
    label_text: String,
  ) -> Handle<'_, Self>
  where
    L: Lens<Target = String>,
  {
    Self { async_executor }.build(cx, |cx| {
      VStack::new(cx, |cx| {
        Label::new(cx, label_text).alignment(Alignment::Center);

        Button::new(cx, |cx| {
          HStack::new(cx, |cx| {
            Label::new(cx, lens)
              .width(Stretch(2.0))
              .text_wrap(false)
              .text_overflow(TextOverflow::Ellipsis)
              .hoverable(false);
            Svg::new(cx, ICON_CHEVRON_DOWN)
              .class("icon")
              .size(Pixels(16.0))
              .hoverable(false);
          })
          .width(Stretch(1.0))
          .gap(Pixels(8.0))
        })
        .class("filedrop")
        .on_drop(|cx, data| {
          if let DropData::File(path_buf) = data {
            cx.emit(ParamFileDropEvent::SetFilePath(path_buf));
          }
        })
        .on_press(|cx| cx.emit(ParamFileDropEvent::PickFileFromDialog));
      })
      .alignment(Alignment::Center)
      .vertical_gap(Pixels(8.0));
    })
  }
}

impl View for ParamFileDrop {
  fn element(&self) -> Option<&'static str> {
    Some("param-filedrop")
  }

  fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
    event.map(|param_event, meta| match param_event {
      ParamFileDropEvent::SetFilePath(path_buf) => {
        self
          .async_executor
          .execute_background(WorkerRequest::LoadFile(
            path_buf.to_string_lossy().into_owned(),
            true,
          ));

        meta.consume();
      }

      ParamFileDropEvent::PickFileFromDialog => {
        cx.spawn(move |cx_proxy| {
          if let Some(path_buf) = FileDialog::new().add_filter("wav", &["wav"]).pick_file() {
            cx_proxy
              .emit(ParamFileDropEvent::SetFilePath(path_buf))
              .ok();
          }
        });

        meta.consume();
      }
    });
  }
}
