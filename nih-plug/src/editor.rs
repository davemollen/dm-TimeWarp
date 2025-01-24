#[path = "./editor/components/param_checkbox.rs"]
mod param_checkbox;
use param_checkbox::ParamCheckbox;
#[path = "./editor/components/param_knob.rs"]
mod param_knob;
use param_knob::{ParamKnob, ParamKnobSize};
mod ui_data;
use crate::grain_stretch_parameters::GrainStretchParameters;
use nih_plug::{params::Param, prelude::Editor};
use nih_plug_vizia::vizia::{
  layout::Units::Auto,
  model::Model,
  modifiers::{LayoutModifiers, StyleModifiers, TextModifiers},
  prelude::Units::{Pixels, Stretch},
  style::FontWeightKeyword,
  views::{Element, HStack, Label, VStack},
};
use nih_plug_vizia::{create_vizia_editor, ViziaState, ViziaTheming};
use std::sync::Arc;
pub use ui_data::{ParamChangeEvent, UiData};

const STYLE: &str = include_str!("./editor/style.css");

// Makes sense to also define this here, makes it a bit easier to keep track of
pub(crate) fn default_state() -> Arc<ViziaState> {
  ViziaState::new(|| (568, 344))
}

pub(crate) fn create(
  params: Arc<GrainStretchParameters>,
  editor_state: Arc<ViziaState>,
) -> Option<Box<dyn Editor>> {
  create_vizia_editor(
    editor_state,
    ViziaTheming::Custom,
    move |cx, gui_context| {
      let _ = cx.add_stylesheet(STYLE);

      UiData {
        params: params.clone(),
        gui_context: gui_context.clone(),
      }
      .build(cx);

      VStack::new(cx, |cx| {
        HStack::new(cx, |cx| {
          VStack::new(cx, |cx| {
            HStack::new(cx, |cx| {
              ParamKnob::new(
                cx,
                params.scan.name(),
                UiData::params,
                params.scan.as_ptr(),
                |params| &params.scan,
                |param_ptr, val| ParamChangeEvent::SetParam(param_ptr, val),
                ParamKnobSize::Regular,
              );
              ParamKnob::new(
                cx,
                params.spray.name(),
                UiData::params,
                params.spray.as_ptr(),
                |params| &params.spray,
                |param_ptr, val| ParamChangeEvent::SetParam(param_ptr, val),
                ParamKnobSize::Regular,
              );
              ParamKnob::new(
                cx,
                params.size.name(),
                UiData::params,
                params.size.as_ptr(),
                |params| &params.size,
                |param_ptr, val| ParamChangeEvent::SetParam(param_ptr, val),
                ParamKnobSize::Regular,
              );
              ParamKnob::new(
                cx,
                params.speed.name(),
                UiData::params,
                params.speed.as_ptr(),
                |params| &params.speed,
                |param_ptr, val| ParamChangeEvent::SetParam(param_ptr, val),
                ParamKnobSize::Large,
              );
              ParamKnob::new(
                cx,
                params.density.name(),
                UiData::params,
                params.density.as_ptr(),
                |params| &params.density,
                |param_ptr, val| ParamChangeEvent::SetParam(param_ptr, val),
                ParamKnobSize::Regular,
              );
              ParamKnob::new(
                cx,
                params.stretch.name(),
                UiData::params,
                params.stretch.as_ptr(),
                |params| &params.stretch,
                |param_ptr, val| ParamChangeEvent::SetParam(param_ptr, val),
                ParamKnobSize::Large,
              );
            })
            .size(Auto)
            .child_space(Stretch(1.0));

            HStack::new(cx, |cx| {
              ParamCheckbox::new(
                cx,
                params.record.name(),
                UiData::params,
                params.record.as_ptr(),
                |params| &params.record,
                |param_ptr, val| ParamChangeEvent::SetParam(param_ptr, val),
              )
              .top(Pixels(12.))
              .width(Pixels(72.));
              ParamKnob::new(
                cx,
                params.time.name(),
                UiData::params,
                params.time.as_ptr(),
                |params| &params.time,
                |param_ptr, val| ParamChangeEvent::SetParam(param_ptr, val),
                ParamKnobSize::Regular,
              );
              ParamKnob::new(
                cx,
                params.highpass.name(),
                UiData::params,
                params.highpass.as_ptr(),
                |params| &params.highpass,
                |param_ptr, val| ParamChangeEvent::SetParam(param_ptr, val),
                ParamKnobSize::Regular,
              );
              ParamKnob::new(
                cx,
                params.lowpass.name(),
                UiData::params,
                params.lowpass.as_ptr(),
                |params| &params.lowpass,
                |param_ptr, val| ParamChangeEvent::SetParam(param_ptr, val),
                ParamKnobSize::Regular,
              );
              ParamKnob::new(
                cx,
                params.overdub.name(),
                UiData::params,
                params.overdub.as_ptr(),
                |params| &params.overdub,
                |param_ptr, val| ParamChangeEvent::SetParam(param_ptr, val),
                ParamKnobSize::Large,
              );
              ParamKnob::new(
                cx,
                params.recycle.name(),
                UiData::params,
                params.recycle.as_ptr(),
                |params| &params.recycle,
                |param_ptr, val| ParamChangeEvent::SetParam(param_ptr, val),
                ParamKnobSize::Regular,
              );
            })
            .size(Auto)
            .child_space(Stretch(1.0));
          })
          .top(Pixels(32.))
          .size(Auto);

          Element::new(cx).class("line");

          VStack::new(cx, |cx| {
            ParamKnob::new(
              cx,
              params.dry.name(),
              UiData::params,
              params.dry.as_ptr(),
              |params| &params.dry,
              |param_ptr, val| ParamChangeEvent::SetParam(param_ptr, val),
              ParamKnobSize::Large,
            );
            ParamKnob::new(
              cx,
              params.wet.name(),
              UiData::params,
              params.wet.as_ptr(),
              |params| &params.wet,
              |param_ptr, val| ParamChangeEvent::SetParam(param_ptr, val),
              ParamKnobSize::Large,
            );
          })
          .size(Auto);
        })
        .col_between(Pixels(16.0));

        Label::new(cx, "dm-GrainStretch")
          .font_size(22.0)
          .font_weight(FontWeightKeyword::Bold)
          .border_radius(Pixels(16.0))
          .border_width(Pixels(1.))
          .border_color("#2c5494")
          .background_color("#3c6ab5")
          .child_top(Pixels(4.0))
          .child_bottom(Pixels(8.0))
          .child_left(Pixels(12.0))
          .child_right(Pixels(12.0))
          .left(Stretch(1.0));
      })
      .child_space(Pixels(16.0))
      .background_color("#505050");
    },
  )
}
