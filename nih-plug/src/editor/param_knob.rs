#[path = "param_knob/arc_track.rs"]
mod arc_track;
use arc_track::ArcTrack;
use nih_plug::params::Param;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::param_base::ParamWidgetBase;

enum ParamKnobEvent {
  SetValue(f32),
  TextInput(String),
}

pub struct ParamKnob {
  param_base: ParamWidgetBase,
}

impl ParamKnob {
  pub fn new<L, Params, P, FMap>(
    cx: &mut Context,
    params: L,
    params_to_param: FMap,
  ) -> Handle<'_, Self>
  where
    L: Lens<Target = Params> + Clone,
    Params: 'static,
    P: Param + 'static,
    FMap: Fn(&Params) -> &P + Copy + 'static,
  {
    Self {
      param_base: ParamWidgetBase::new(cx, params, params_to_param),
    }
    .build(
      cx,
      ParamWidgetBase::build_view(params, params_to_param, |cx, param_data| {
        let unmodulated_normalized_value_lens =
          param_data.make_lens(|param| param.modulated_normalized_value());
        let display_value_lens = param_data.make_lens(|param| {
          param.normalized_value_to_string(param.modulated_normalized_value(), true)
        });
        let default_normalized_value = param_data.param().default_normalized_value();

        VStack::new(cx, |cx| {
          Label::new(cx, param_data.param().name())
            .font_size(11.0)
            .font_weight(FontWeightKeyword::SemiBold)
            .child_space(Stretch(1.0));

          Knob::custom(
            cx,
            default_normalized_value,
            unmodulated_normalized_value_lens,
            |cx, lens| {
              ZStack::new(cx, |cx| {
                ArcTrack::new(
                  cx,
                  lens,
                  false,
                  Percentage(100.0),
                  Percentage(15.0),
                  -150.,
                  150.,
                  KnobMode::Continuous,
                )
                .class("knob-track");

                HStack::new(cx, |cx| {
                  Element::new(cx).class("knob-tick");
                })
                .rotate(lens.map(|v| Angle::Deg(*v * 300.0 - 150.0)))
                .class("knob-head");
              })
            },
          )
          .size(Pixels(48.0))
          .on_changing(|cx, val| cx.emit(ParamKnobEvent::SetValue(val)));

          Textbox::new(cx, display_value_lens)
            .placeholder("..")
            .on_mouse_down(|cx, _| {
              if cx.is_disabled() {
                return;
              }
              cx.emit(TextEvent::StartEdit);
              cx.emit(TextEvent::Clear);
            })
            .on_submit(|cx, text, success| {
              cx.emit(TextEvent::EndEdit);
              if success {
                cx.emit(ParamKnobEvent::TextInput(text));
              };
            })
            .font_size(10.0)
            .top(Pixels(-1.0))
            .text_align(TextAlign::Center);
        })
        .size(Auto)
        .child_space(Stretch(1.0))
        .row_between(Pixels(3.0));
      }),
    )
  }
}

impl View for ParamKnob {
  fn element(&self) -> Option<&'static str> {
    Some("param-knob")
  }

  fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
    event.map(|param_event, meta| match param_event {
      ParamKnobEvent::SetValue(val) => {
        self.param_base.begin_set_parameter(cx);
        self.param_base.set_normalized_value(cx, *val);
        self.param_base.end_set_parameter(cx);
        meta.consume();
      }

      ParamKnobEvent::TextInput(string) => {
        if let Some(normalized_value) = self.param_base.string_to_normalized_value(string) {
          self.param_base.begin_set_parameter(cx);
          self.param_base.set_normalized_value(cx, normalized_value);
          self.param_base.end_set_parameter(cx);
        }
        meta.consume();
      }
    });
  }
}
