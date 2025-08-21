use crate::{worker::WorkerRequest, DmTimeWarp};
use nih_plug::prelude::AsyncExecutor;
use nih_plug_vizia::vizia::{icons::ICON_CHEVRON_DOWN, prelude::*};
use rfd::FileDialog;
use std::path::PathBuf;

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
        Label::new(cx, &label_text)
          .font_size(13.0)
          .font_weight(FontWeightKeyword::SemiBold)
          .child_space(Stretch(1.0));

        Button::new(
          cx,
          |cx| cx.emit(ParamFileDropEvent::PickFileFromDialog),
          |cx| {
            Label::new(cx, lens)
              .font_size(10.0)
              .font_weight(FontWeightKeyword::Bold)
              .text_wrap(false)
              .hoverable(false)
              .width(Pixels(72.0))
              .left(Pixels(4.0))
              .class("file-path");
            Label::new(cx, ICON_CHEVRON_DOWN)
              .class("icon")
              .right(Pixels(0.0))
              .hoverable(false)
          },
        )
        .class("filedrop")
        .on_drop(|cx, data| {
          if let DropData::File(path_buf) = data {
            cx.emit(ParamFileDropEvent::SetFilePath(path_buf));
          }
        });
      })
      .size(Auto)
      .child_space(Stretch(1.0))
      .row_between(Pixels(4.0));
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
        if cx.is_disabled() {
          meta.consume();
          return;
        }

        cx.spawn(move |cx_proxy| {
          if let Some(path_buf) = FileDialog::new()
            .add_filter(
              "audio_file",
              &[
                "aac", "aif", "aiff", "caf", "flac", "m4a", "mka", "mkv", "mp1", "mp2", "mp3",
                "mp4", "oga", "ogg", "opus", "raw", "wav", "wv", "webm",
              ],
            )
            .pick_file()
          {
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
