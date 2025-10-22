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
    editor::param_knob::ParamKnobHandle,
    time_warp_parameters::{SampleMode, TimeWarpParameters},
    DmTimeWarp,
  },
  assets::{register_roboto, register_roboto_bold, ROBOTO_FONT_NAME},
  nih_plug::prelude::{AsyncExecutor, Editor, Enum},
  param_button::ParamButton,
  param_file_drop::ParamFileDrop,
  param_footswitch::{ParamFootswitch, ParamFootswitchHandle},
  param_knob::ParamKnob,
  param_number_input::ParamNumberInput,
  param_slider::ParamSlider,
  param_switch::ParamSwitch,
  param_tabs::ParamTabs,
  std::sync::Arc,
  vizia_plug::{
    create_vizia_editor,
    vizia::{
      icons::{ICON_PLAYER_PLAY_FILLED, ICON_PLAYER_RECORD_FILLED, ICON_PLAYER_STOP_FILLED},
      prelude::*,
    },
    ViziaState, ViziaTheming,
  },
};

const STYLE: &str = include_str!("editor/style.css");

#[derive(Lens)]
pub struct Data {
  pub params: Arc<TimeWarpParameters>,
}

impl Model for Data {}

pub(crate) fn default_state() -> Arc<ViziaState> {
  ViziaState::new(|| (936, 344))
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
            HStack::new(cx, |cx| {
              HStack::new(cx, |cx| {
                ParamButton::new(cx, Data::params, |params| &params.freeze).size(Auto);
              })
              .size(Auto)
              .padding_right(Pixels(-4.0));
              ParamKnob::new(cx, Data::params, |params| &params.stretch)
                .disabled(Data::params.map(|p| p.freeze.value()))
                .size(Auto);
            })
            .alignment(Alignment::Center)
            .size(Auto);
            ParamKnob::new(cx, Data::params, |params| &params.size).size(Auto);
            ParamKnob::new(cx, Data::params, |params| &params.density).size(Auto);
            ParamKnob::new(cx, Data::params, |params| &params.stereo).size(Auto);
            HStack::new(cx, |cx| {
              HStack::new(cx, |cx| {
                ParamKnob::new(cx, Data::params, |params| &params.detune)
                  .knob_size(36.)
                  .size(Auto);
                HStack::new(cx, |cx| {
                  Element::new(cx).class("detune-knob-line");
                })
                .alignment(Alignment::Center)
                .padding_left(Pixels(-10.0));
              })
              .size(Auto)
              .padding_left(Pixels(-4.0))
              .padding_right(Pixels(-8.0))
              .alignment(Alignment::Center);
              ParamKnob::new(cx, Data::params, |params| &params.pitch).size(Auto);
            })
            .size(Auto)
            .alignment(Alignment::Center);
          })
          .width(Stretch(1.0))
          .height(Auto)
          .horizontal_gap(Pixels(4.0))
          .alignment(Alignment::Right)
          .border_color("#797979")
          .border_width(Pixels(2.0))
          .corner_top_right_radius(Pixels(8.0))
          .padding(Pixels(12.0))
          .background_color("#211F24");

          HStack::new(cx, |cx| {
            ParamFileDrop::new(
              cx,
              async_executor.clone(),
              Data::params.map(|p| p.file_path.lock().unwrap().clone()),
              "Sample".to_string(),
            )
            .size(Auto)
            .disabled(Data::params.map(|p| p.sample_mode.value() != SampleMode::Sampler));
            ParamTabs::new(cx, SampleMode::variants(), Data::params, |params| {
              &params.sample_mode
            })
            .padding_left(Pixels(8.0))
            .padding_right(Pixels(8.0))
            .size(Auto);

            HStack::new(cx, |cx| {
              HStack::new(cx, |cx| {
                ParamButton::new(cx, Data::params, |params| &params.sync)
                  .size(Auto)
                  .class("show")
                  .disabled(Data::params.map(|p| p.sample_mode.value() != SampleMode::Delay));
              })
              .size(Auto)
              .padding_right(Pixels(-4.0));

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
            .size(Auto)
            .alignment(Alignment::Center);
            ParamKnob::new(cx, Data::params, |params| &params.highpass).size(Auto);
            ParamKnob::new(cx, Data::params, |params| &params.lowpass).size(Auto);
            ParamKnob::new(cx, Data::params, |params| &params.recycle).size(Auto);
            ParamKnob::new(cx, Data::params, |params| &params.feedback).size(Auto);
          })
          .width(Stretch(1.0))
          .height(Auto)
          .horizontal_gap(Pixels(4.0))
          .alignment(Alignment::Right)
          .border_color("#797979")
          .border_width(Pixels(2.0))
          .corner_bottom_right_radius(Pixels(8.0))
          .padding(Pixels(12.0))
          .background_color("#211F24");
        })
        .alignment(Alignment::BottomLeft)
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
                ParamSwitch::new(cx, Data::params, |params| &params.sync_position)
                  .size(Auto)
                  .disabled(Data::params.map(|p| !p.midi_enabled.value()));
                ParamNumberInput::new(cx, Data::params, |params| &params.voices)
                  .size(Auto)
                  .disabled(Data::params.map(|p| !p.midi_enabled.value()));
              })
              .alignment(Alignment::TopCenter)
              .width(Auto)
              .vertical_gap(Pixels(8.0));
            })
            .size(Auto)
            .padding_top(Pixels(16.0))
            .padding_left(Pixels(12.0))
            .padding_right(Pixels(12.0))
            .padding_bottom(Pixels(16.0))
            .horizontal_gap(Pixels(4.0))
            .border_color("#797979")
            .border_width(Pixels(2.0))
            .corner_bottom_left_radius(Pixels(8.0))
            .background_color("#211F24");

            HStack::new(cx, |cx| {
              ParamSlider::new(cx, Data::params, |params| &params.dry).size(Auto);
              ParamSlider::new(cx, Data::params, |params| &params.wet).size(Auto);
            })
            .size(Auto)
            .horizontal_gap(Pixels(4.0))
            .border_color("#797979")
            .border_width(Pixels(2.0))
            .padding_top(Pixels(16.0))
            .padding_left(Pixels(12.0))
            .padding_right(Pixels(12.0))
            .padding_bottom(Pixels(16.0))
            .background_color("#211F24");
          })
          .size(Auto);

          VStack::new(cx, |cx| {
            Element::new(cx).class("logo");
          })
          .size(Stretch(1.0))
          .alignment(Alignment::Center);
        })
        .width(Auto)
        .height(Stretch(1.0));
      });

      HStack::new(cx, |cx| {
        ParamFootswitch::new(
          cx,
          |cx| {
            HStack::new(cx, |cx| {
              Svg::new(cx, ICON_PLAYER_RECORD_FILLED)
                .fill("#ececec")
                .size(Pixels(12.5));
              Label::new(cx, " / Dub")
                .font_size(11.0)
                .font_weight(FontWeightKeyword::SemiBold);
            })
            .alignment(Alignment::Center)
            .size(Auto);
          },
          Data::params,
          |params| &params.record,
        )
        .size(Auto);
        ParamFootswitch::new(
          cx,
          |cx| {
            HStack::new(cx, |cx| {
              Svg::new(cx, ICON_PLAYER_PLAY_FILLED)
                .fill("#ececec")
                .size(Pixels(12.5));
              Label::new(cx, "  /  ")
                .font_size(11.0)
                .font_weight(FontWeightKeyword::SemiBold);
              Svg::new(cx, ICON_PLAYER_STOP_FILLED)
                .fill("#ececec")
                .size(Pixels(12.5));
            })
            .alignment(Alignment::Center)
            .size(Auto);
          },
          Data::params,
          |params| &params.play,
        )
        .size(Auto);
        ParamFootswitch::new(
          cx,
          |cx| {
            Label::new(cx, "Erase")
              .font_size(11.0)
              .font_weight(FontWeightKeyword::SemiBold);
          },
          Data::params,
          |params| &params.erase,
        )
        .is_momentary(true)
        .size(Auto);
      })
      .height(Auto)
      .padding_top(Pixels(16.0))
      .padding_bottom(Pixels(16.0))
      .padding_left(Percentage(14.0))
      .padding_right(Percentage(14.0))
      .border_color("#797979")
      .border_width(Pixels(2.0))
      .background_color("#211F24")
      .horizontal_gap(Stretch(1.0));
    })
    .class("background");
  })
}
