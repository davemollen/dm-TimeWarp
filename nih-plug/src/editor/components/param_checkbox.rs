use nih_plug::prelude::{Param, ParamPtr};
use nih_plug_vizia::vizia::{
  binding::Lens,
  layout::Units::Auto,
  modifiers::TextModifiers,
  prelude::{
    ActionModifiers, Context, EmitContext, LayoutModifiers, LensExt,
    Units::{Pixels, Stretch},
  },
  style::FontWeightKeyword,
  view::Handle,
  views::{Checkbox, Label, VStack},
};
use std::any::Any;

pub struct ParamCheckbox;

impl ParamCheckbox {
  pub fn new<'a, L, P, F, M, C>(
    cx: &'a mut Context,
    name: &'a str,
    lens: L,
    param_ptr: ParamPtr,
    params_to_param: F,
    on_change: C,
  ) -> Handle<'a, VStack>
  where
    L: 'static + Lens + Copy + Send + Sync,
    <L as Lens>::Source: 'static,
    P: Param<Plain = bool>,
    F: 'static + Fn(&<L as Lens>::Target) -> &P + Copy + Send + Sync,
    M: Any + Send,
    C: 'static + Fn(ParamPtr, f32) -> M + Copy + Send + Sync,
  {
    VStack::new(cx, |cx| {
      Label::new(cx, name)
        .font_size(13.0)
        .font_weight(FontWeightKeyword::SemiBold)
        .child_space(Stretch(1.0));

      Checkbox::new(
        cx,
        lens.map(move |p: &<L as Lens>::Target| params_to_param(p).modulated_plain_value()),
      )
      .on_press(move |cx| {
        let current_normalized_value = lens
          .map(move |params| params_to_param(params).modulated_normalized_value())
          .get(cx);

        cx.emit(on_change(param_ptr, 1. - current_normalized_value));
      });
    })
    .size(Auto)
    .child_left(Stretch(1.0))
    .child_right(Stretch(1.0))
    .row_between(Pixels(8.0))
  }
}
