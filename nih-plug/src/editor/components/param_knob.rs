use nih_plug::prelude::{Param, ParamPtr};
use nih_plug_vizia::vizia::{
  binding::Lens,
  layout::Units::Auto,
  modifiers::TextModifiers,
  prelude::{
    ActionModifiers, Context, EmitContext, LayoutModifiers, LensExt, Units,
    Units::{Pixels, Stretch},
  },
  style::FontWeightKeyword,
  view::Handle,
  views::{Knob, Label, TextEvent, Textbox, VStack},
};
use std::any::Any;

pub enum ParamKnobSize {
  Small,
  Regular,
  Large,
}

impl ParamKnobSize {
  fn get_value(&self) -> Units {
    match self {
      ParamKnobSize::Small => Pixels(32.),
      ParamKnobSize::Regular => Pixels(44.),
      ParamKnobSize::Large => Pixels(68.),
    }
  }
}

pub struct ParamKnob;

impl ParamKnob {
  pub fn new<'a, L, P, F, M, C>(
    cx: &'a mut Context,
    name: &'a str,
    lens: L,
    param_ptr: ParamPtr,
    params_to_param: F,
    on_change: C,
    size: ParamKnobSize,
  ) -> Handle<'a, VStack>
  where
    L: 'static + Lens + Copy + Send + Sync,
    <L as Lens>::Source: 'static,
    P: Param,
    F: 'static + Fn(&<L as Lens>::Target) -> &P + Copy + Send + Sync,
    M: Any + Send,
    C: 'static + Fn(ParamPtr, f32) -> M + Copy + Send + Sync,
  {
    VStack::new(cx, |cx| {
      Label::new(cx, name)
        .font_size(13.0)
        .font_weight(FontWeightKeyword::SemiBold)
        .text_wrap(false)
        .child_space(Stretch(1.0));

      Knob::new(
        cx,
        lens.map(move |p| params_to_param(p).default_normalized_value()),
        lens.map(move |p| params_to_param(p).modulated_normalized_value()),
        false,
      )
      .on_changing(move |cx, val| {
        cx.emit(on_change(param_ptr, val));
      })
      .size(size.get_value());

      Textbox::new(
        cx,
        lens.map(move |p| {
          params_to_param(p)
            .normalized_value_to_string(params_to_param(p).modulated_normalized_value(), true)
        }),
      )
      .on_mouse_down(|cx, _| {
        cx.emit(TextEvent::StartEdit);
        cx.emit(TextEvent::Clear);
      })
      .on_submit(move |cx, text, success| {
        cx.emit(TextEvent::EndEdit);

        if success {
          let val = lens
            .map(move |p| params_to_param(p).string_to_normalized_value(&text))
            .get(cx);
          match val {
            Some(val) => cx.emit(on_change(param_ptr, val)),
            _ => (),
          };
        }
      })
      .font_size(12.0)
      .top(Pixels(-4.0));
    })
    .size(Auto)
    .child_space(Stretch(1.0))
    .row_between(Pixels(4.0))
  }
}
