use nih_plug::params::Param;
use vizia_plug::{vizia::prelude::*, widgets::param_base::ParamWidgetBase};

enum ParamButtonEvent {
  Toggle,
}
pub struct ParamButton {
  param_base: ParamWidgetBase,
}

impl ParamButton {
  pub fn new<L, Params, P, FMap>(
    cx: &mut Context,
    params: L,
    params_to_param: FMap,
  ) -> Handle<'_, Self>
  where
    L: Lens<Target = Params> + Clone,
    Params: 'static,
    P: Param<Plain = bool> + 'static,
    FMap: Fn(&Params) -> &P + Copy + 'static,
  {
    Self {
      param_base: ParamWidgetBase::new(cx, params, params_to_param),
    }
    .build(
      cx,
      ParamWidgetBase::build_view(params, params_to_param, |cx, param_data| {
        let value = param_data.make_lens(|param| param.modulated_plain_value());
        let label_text = param_data.param().name();

        HStack::new(cx, |cx| {
          Label::new(cx, &label_text.to_uppercase())
            .checkable(true)
            .checked(value)
            .on_press(|cx| cx.emit(ParamButtonEvent::Toggle))
            .font_size(9.0)
            .font_weight(FontWeightKeyword::Bold)
            .alignment(Alignment::Center)
            .padding_left(Pixels(4.0))
            .padding_right(Pixels(4.0));
          Element::new(cx).class("line");
        })
        .size(Auto)
        .alignment(Alignment::Center);
      }),
    )
  }
}

impl View for ParamButton {
  fn element(&self) -> Option<&'static str> {
    Some("param-button2")
  }

  fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
    event.map(|param_event, meta| match param_event {
      ParamButtonEvent::Toggle => {
        let normalized_value = self.param_base.modulated_normalized_value();
        self.param_base.begin_set_parameter(cx);
        self
          .param_base
          .set_normalized_value(cx, 1. - normalized_value);
        self.param_base.end_set_parameter(cx);

        meta.consume();
      }
    });
  }
}
