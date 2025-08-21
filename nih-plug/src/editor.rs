mod assets;
mod param_button;
mod param_file_drop;
mod param_footswitch;
mod param_knob;
mod param_number_input;
mod param_slider;
mod param_switch;
mod param_tabs;
use {
  crate::{
    time_warp_parameters::{SampleMode, TimeWarpParameters},
    DmTimeWarp,
  },
  assets::{register_roboto, register_roboto_bold, ROBOTO_FONT_NAME},
  nih_plug::prelude::{AsyncExecutor, Editor, Enum},
  nih_plug_vizia::{
    create_vizia_editor,
    vizia::{image, prelude::*},
    ViziaState, ViziaTheming,
  },
  param_button::ParamButton,
  param_file_drop::ParamFileDrop,
  param_footswitch::{ParamFootswitch, ParamFootswitchHandle},
  param_knob::ParamKnob,
  param_number_input::ParamNumberInput,
  param_slider::ParamSlider,
  param_switch::ParamSwitch,
  param_tabs::ParamTabs,
  std::sync::Arc,
};

const STYLE: &str = include_str!("editor/style.css");

#[derive(Lens)]
pub struct Data {
  pub params: Arc<TimeWarpParameters>,
}

impl Model for Data {}

pub(crate) fn default_state() -> Arc<ViziaState> {
  ViziaState::new(|| (1156, 426))
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
      image::load_from_memory_with_format(
        include_bytes!("editor/assets/background.png"),
        image::ImageFormat::Png,
      )
      .unwrap(),
      ImageRetentionPolicy::DropWhenUnusedForOneFrame,
    );
    cx.load_image(
      "logo.png",
      image::load_from_memory_with_format(
        include_bytes!("editor/assets/logo.png"),
        image::ImageFormat::Png,
      )
      .unwrap(),
      ImageRetentionPolicy::DropWhenUnusedForOneFrame,
    );

    Data {
      params: params.clone(),
    }
    .build(cx);

    VStack::new(cx, |cx| {
      HStack::new(cx, |cx| {
        VStack::new(cx, |cx| {
          HStack::new(cx, |cx| {
            ParamKnob::new(cx, Data::params, |params| &params.scan).size(Auto);
            ParamKnob::new(cx, Data::params, |params| &params.spray).size(Auto);
            ParamKnob::new(cx, Data::params, |params| &params.size).size(Auto);
            ParamKnob::new(cx, Data::params, |params| &params.density).size(Auto);
            ParamKnob::new(cx, Data::params, |params| &params.stereo).size(Auto);
            ParamKnob::new(cx, Data::params, |params| &params.pitch).size(Auto);
            ParamKnob::new(cx, Data::params, |params| &params.stretch).size(Auto);
          })
          .width(Stretch(1.0))
          .height(Auto)
          .col_between(Pixels(16.0))
          .border_color("#797979")
          .border_width(Pixels(2.0))
          .border_top_right_radius(Pixels(8.0))
          .top(Stretch(1.0))
          .child_space(Pixels(16.0))
          .child_left(Stretch(1.0))
          .background_color("#211F24");

          HStack::new(cx, |cx| {
            ParamFileDrop::new(
              cx,
              async_executor.clone(),
              Data::params.map(|p| p.file_path.lock().unwrap().clone()),
              "Sample".to_string(),
            )
            .size(Auto)
            .top(Stretch(1.0))
            .bottom(Stretch(1.0))
            .disabled(Data::params.map(|p| p.sample_mode.value() != SampleMode::Sampler));
            ParamTabs::new(cx, SampleMode::variants(), Data::params, |params| {
              &params.sample_mode
            })
            .size(Auto)
            .top(Stretch(1.0))
            .bottom(Stretch(1.0))
            .right(Pixels(48.0));

            VStack::new(cx, |cx| {
              ParamButton::new(cx, Data::params, |params| &params.sync)
                .size(Auto)
                .class("show")
                .disabled(Data::params.map(|p| p.sample_mode.value() != SampleMode::Delay));
              ParamKnob::new(cx, Data::params, |params| &params.time)
                .size(Auto)
                .class("show")
                .toggle_class(
                  "hide",
                  Data::params
                    .map(|p| p.sync.value() || p.sample_mode.value() != SampleMode::Delay),
                );
              ParamKnob::new(cx, Data::params, |params| &params.division)
                .size(Auto)
                .class("show")
                .toggle_class(
                  "hide",
                  Data::params
                    .map(|p| !p.sync.value() || p.sample_mode.value() != SampleMode::Delay),
                );
              ParamKnob::new(cx, Data::params, |params| &params.length)
                .size(Auto)
                .class("show")
                .toggle_class(
                  "hide",
                  Data::params.map(|p| p.sample_mode.value() == SampleMode::Delay),
                );
            })
            .size(Auto);
            ParamKnob::new(cx, Data::params, |params| &params.highpass).size(Auto);
            ParamKnob::new(cx, Data::params, |params| &params.lowpass).size(Auto);
            ParamKnob::new(cx, Data::params, |params| &params.recycle).size(Auto);
            ParamKnob::new(cx, Data::params, |params| &params.feedback).size(Auto);
          })
          .width(Stretch(1.0))
          .height(Auto)
          .col_between(Pixels(16.0))
          .border_color("#797979")
          .border_width(Pixels(2.0))
          .border_bottom_right_radius(Pixels(8.0))
          .child_space(Pixels(16.0))
          .child_left(Stretch(1.0))
          .background_color("#211F24");
        })
        .width(Stretch(1.0));

        VStack::new(cx, |cx| {
          HStack::new(cx, |cx| {
            HStack::new(cx, |cx| {
              ParamSlider::new(cx, Data::params, |params| &params.attack)
                .size(Auto)
                .disabled(Data::params.map(|p| !p.midi_enabled.value()));
              ParamSlider::new(cx, Data::params, |params| &params.decay)
                .size(Auto)
                .disabled(Data::params.map(|p| !p.midi_enabled.value()));
              ParamSlider::new(cx, Data::params, |params| &params.sustain)
                .size(Auto)
                .disabled(Data::params.map(|p| !p.midi_enabled.value()));
              ParamSlider::new(cx, Data::params, |params| &params.release)
                .size(Auto)
                .disabled(Data::params.map(|p| !p.midi_enabled.value()));
              VStack::new(cx, |cx| {
                ParamSwitch::new(cx, Data::params, |params| &params.midi_enabled).size(Auto);
                ParamNumberInput::new(cx, Data::params, |params| &params.voices)
                  .size(Auto)
                  .top(Stretch(1.0))
                  .disabled(Data::params.map(|p| !p.midi_enabled.value()));
              })
              .child_left(Stretch(1.0))
              .child_right(Stretch(1.0))
              .width(Auto)
              .height(Stretch(1.0));
            })
            .size(Auto)
            .child_space(Pixels(16.0))
            .col_between(Pixels(4.0))
            .border_color("#797979")
            .border_width(Pixels(2.0))
            .border_bottom_left_radius(Pixels(8.0))
            .background_color("#211F24");

            HStack::new(cx, |cx| {
              ParamSlider::new(cx, Data::params, |params| &params.dry).size(Auto);
              ParamSlider::new(cx, Data::params, |params| &params.wet).size(Auto);
            })
            .size(Auto)
            .col_between(Pixels(4.0))
            .border_color("#797979")
            .border_width(Pixels(2.0))
            .child_space(Pixels(16.0))
            .background_color("#211F24");
          })
          .size(Auto);
          Element::new(cx).class("logo");
        })
        .size(Auto)
        .right(Pixels(0.0));
      });
      HStack::new(cx, |cx| {
        ParamFootswitch::new(cx, Data::params, |params| &params.record)
          .size(Auto)
          .left(Stretch(1.0))
          .right(Stretch(1.0));
        ParamFootswitch::new(cx, Data::params, |params| &params.play)
          .size(Auto)
          .left(Stretch(1.0))
          .right(Stretch(1.0));
        ParamFootswitch::new(cx, Data::params, |params| &params.erase)
          .is_momentary(true)
          .size(Auto)
          .left(Stretch(1.0))
          .right(Stretch(1.0));
      })
      .width(Stretch(1.0))
      .height(Auto)
      .child_space(Pixels(16.0))
      .border_color("#797979")
      .border_width(Pixels(2.0))
      .col_between(Pixels(4.0))
      .background_color("#211F24");
    })
    .class("background");
  })
}
