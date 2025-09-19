use nih_plug::params::Param;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::param_base::ParamWidgetBase;

enum ParamSliderEvent {
  SetValue(f32),
  TextInput(String),
}

pub struct ParamSlider {
  param_base: ParamWidgetBase,
}

impl ParamSlider {
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

        VStack::new(cx, |cx| {
          Label::new(cx, param_data.param().name())
            .font_size(11.0)
            .font_weight(FontWeightKeyword::SemiBold)
            .child_space(Stretch(1.0));
          Slider::new(cx, unmodulated_normalized_value_lens)
            .on_changing(|cx, val| cx.emit(ParamSliderEvent::SetValue(val)))
            .class("vertical");
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
                cx.emit(ParamSliderEvent::TextInput(text));
              };
            })
            .font_size(10.0)
            .text_align(TextAlign::Center);
        })
        .size(Auto)
        .child_space(Stretch(1.0))
        .row_between(Pixels(3.0));
      }),
    )
  }
}

impl View for ParamSlider {
  fn element(&self) -> Option<&'static str> {
    Some("param-slider2")
  }

  fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
    event.map(|param_event, meta| match param_event {
      ParamSliderEvent::SetValue(val) => {
        self.param_base.begin_set_parameter(cx);
        self.param_base.set_normalized_value(cx, *val);
        self.param_base.end_set_parameter(cx);
        meta.consume();
      }

      ParamSliderEvent::TextInput(string) => {
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
