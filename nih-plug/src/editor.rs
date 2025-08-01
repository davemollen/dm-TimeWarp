#[path = "./editor/components/param_footswitch.rs"]
mod param_footswitch;
use param_footswitch::ParamFootswitch;
#[path = "./editor/components/param_knob.rs"]
mod param_knob;
use param_knob::ParamKnob;
#[path = "./editor/components/param_slider.rs"]
mod param_slider;
use param_slider::ParamSlider;
#[path = "./editor/components/param_switch.rs"]
mod param_switch;
use param_switch::ParamSwitch;
#[path = "./editor/components/param_number_input.rs"]
mod param_number_input;
use param_number_input::ParamNumberInput;
#[path = "./editor/components/param_tabs.rs"]
mod param_tabs;
use param_tabs::ParamTabs;
#[path = "./editor/components/param_file_drop.rs"]
mod param_file_drop;
use param_file_drop::ParamFileDrop;
mod assets;
use crate::time_warp_parameters::RecordMode;
use crate::{time_warp_parameters::TimeWarpParameters, DmTimeWarp};
use assets::{register_roboto, register_roboto_bold, ROBOTO_FONT_NAME};
use nih_plug::prelude::{AsyncExecutor, Editor, Enum};
use std::sync::Arc;
use vizia_plug::vizia::prelude::*;
use vizia_plug::{create_vizia_editor, ViziaState, ViziaTheming};

const STYLE: &str = include_str!("editor/style.css");

#[derive(Lens)]
pub struct Data {
  pub params: Arc<TimeWarpParameters>,
}

impl Model for Data {}

pub(crate) fn default_state() -> Arc<ViziaState> {
  ViziaState::new(|| (1114, 426))
}

pub(crate) fn create(
  params: Arc<TimeWarpParameters>,
  editor_state: Arc<ViziaState>,
  async_executor: AsyncExecutor<DmTimeWarp>,
) -> Option<Box<dyn Editor>> {
  create_vizia_editor(editor_state, ViziaTheming::Custom, move |cx, _| {
    register_roboto(cx);
    register_roboto_bold(cx);
    cx.set_default_font(&[ROBOTO_FONT_NAME]);
    cx.add_stylesheet(STYLE).ok();
    cx.load_image(
      "background.png",
      include_bytes!("editor/assets/background.png"),
      ImageRetentionPolicy::DropWhenNoObservers,
    );
    cx.load_image(
      "logo.png",
      include_bytes!("editor/assets/logo.png"),
      ImageRetentionPolicy::DropWhenNoObservers,
    );
    cx.scale_factor();

    Data {
      params: params.clone(),
    }
    .build(cx);

    VStack::new(cx, |cx| {
      HStack::new(cx, |cx| {
        VStack::new(cx, |cx| {
          HStack::new(cx, |cx| {
            ParamKnob::new(cx, Data::params, |params| &params.scan);
            ParamKnob::new(cx, Data::params, |params| &params.spray);
            ParamKnob::new(cx, Data::params, |params| &params.size);
            ParamKnob::new(cx, Data::params, |params| &params.speed);
            ParamKnob::new(cx, Data::params, |params| &params.density);
            ParamKnob::new(cx, Data::params, |params| &params.stretch);
          })
          .width(Pixels(658.0))
          .height(Pixels(114.0))
          .border_color("#797979")
          .border_width(Pixels(2.0))
          .corner_top_right_radius(Pixels(8.0))
          .top(Pixels(64.0))
          .padding(Pixels(16.0))
          .horizontal_gap(Pixels(16.0))
          .background_color("#211F24");

          HStack::new(cx, |cx| {
            ParamFileDrop::new(
              cx,
              async_executor.clone(),
              Data::params.map(|p| p.file_path.lock().unwrap().clone()),
              "Sample".to_string(),
            );
            ParamTabs::new(cx, RecordMode::variants(), Data::params, |params| {
              &params.record_mode
            });
            ParamKnob::new(cx, Data::params, |params| &params.time);
            ParamKnob::new(cx, Data::params, |params| &params.highpass);
            ParamKnob::new(cx, Data::params, |params| &params.lowpass);
            ParamKnob::new(cx, Data::params, |params| &params.feedback);
            ParamKnob::new(cx, Data::params, |params| &params.recycle);
          })
          .width(Pixels(658.0))
          .height(Pixels(114.0))
          .border_color("#797979")
          .border_width(Pixels(2.0))
          .corner_bottom_right_radius(Pixels(8.0))
          .padding(Pixels(16.0))
          .horizontal_gap(Pixels(16.0))
          .background_color("#211F24");
        });
        VStack::new(cx, |cx| {
          HStack::new(cx, |cx| {
            HStack::new(cx, |cx| {
              ParamSlider::new(cx, Data::params, |params| &params.attack);
              ParamSlider::new(cx, Data::params, |params| &params.decay);
              ParamSlider::new(cx, Data::params, |params| &params.sustain);
              ParamSlider::new(cx, Data::params, |params| &params.release);
              VStack::new(cx, |cx| {
                ParamSwitch::new(cx, Data::params, |params| &params.midi_enabled);
                ParamNumberInput::new(cx, Data::params, |params| &params.voices);
              });
            })
            .width(Pixels(318.0))
            .height(Pixels(213.0))
            .border_color("#797979")
            .border_width(Pixels(2.0))
            .corner_bottom_left_radius(Pixels(8.0))
            .padding(Pixels(16.0))
            .horizontal_gap(Pixels(16.0))
            .background_color("#211F24");

            HStack::new(cx, |cx| {
              ParamSlider::new(cx, Data::params, |params| &params.dry);
              ParamSlider::new(cx, Data::params, |params| &params.wet);
            })
            .width(Pixels(138.0))
            .height(Pixels(213.0))
            .border_color("#797979")
            .border_width(Pixels(2.0))
            .padding(Pixels(16.0))
            .horizontal_gap(Pixels(16.0))
            .background_color("#211F24");
          });
          Element::new(cx)
            .width(Pixels(458.0))
            .height(Pixels(77.0))
            .class("logo");
        });
      });
      HStack::new(cx, |cx| {
        ParamFootswitch::new(cx, Data::params, |params| &params.record);
        ParamFootswitch::new(cx, Data::params, |params| &params.play);
        ParamFootswitch::new(cx, Data::params, |params| &params.erase);
      })
      .height(Pixels(136.))
      .border_color("#797979")
      .border_width(Pixels(2.0))
      .horizontal_gap(Pixels(16.0))
      .background_color("#211F24");
    })
    .class("background");
  })
}
