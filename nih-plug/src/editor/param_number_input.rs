use nih_plug::params::Param;
use nih_plug_vizia::{vizia::prelude::*, widgets::param_base::ParamWidgetBase};

enum ParamNumberInputEvent {
  TextInput(String),
}

pub struct ParamNumberInput {
  param_base: ParamWidgetBase,
}

impl ParamNumberInput {
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
        let display_value_lens = param_data.make_lens(|param| {
          param.normalized_value_to_string(param.unmodulated_normalized_value(), true)
        });

        VStack::new(cx, |cx| {
          Label::new(cx, param_data.param().name())
            .font_size(13.0)
            .font_weight(FontWeightKeyword::SemiBold)
            .child_space(Stretch(1.0));
          Textbox::new(cx, display_value_lens)
            .placeholder("..")
            .on_mouse_down(|cx, _| {
              cx.emit(TextEvent::StartEdit);
              cx.emit(TextEvent::Clear);
            })
            .on_submit(|cx, text, success| {
              cx.emit(TextEvent::EndEdit);
              if success {
                cx.emit(ParamNumberInputEvent::TextInput(text));
              };
            })
            .text_align(TextAlign::Center);
        })
        .size(Auto)
        .child_space(Stretch(1.0))
        .row_between(Pixels(4.0));
      }),
    )
  }
}

impl View for ParamNumberInput {
  fn element(&self) -> Option<&'static str> {
    Some("param-number-input")
  }

  fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
    event.map(|param_event, meta| match param_event {
      ParamNumberInputEvent::TextInput(string) => {
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
