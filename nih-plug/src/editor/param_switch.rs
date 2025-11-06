use nih_plug::params::Param;
use nih_plug_vizia::{vizia::prelude::*, widgets::param_base::ParamWidgetBase};

enum ParamSwitchEvent {
  Toggle,
}
pub struct ParamSwitch {
  param_base: ParamWidgetBase,
}

impl ParamSwitch {
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

        VStack::new(cx, |cx| {
          Label::new(cx, label_text)
            .describing(label_text)
            .font_size(11.0)
            .font_weight(FontWeightKeyword::SemiBold)
            .child_space(Stretch(1.0));
          Switch::new(cx, value)
            .on_toggle(|cx| cx.emit(ParamSwitchEvent::Toggle))
            .id(label_text);
        })
        .size(Auto)
        .row_between(Pixels(3.0))
        .child_space(Stretch(1.0));
      }),
    )
  }
}

impl View for ParamSwitch {
  fn element(&self) -> Option<&'static str> {
    Some("param-switch")
  }

  fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
    event.map(|param_event, meta| match param_event {
      ParamSwitchEvent::Toggle => {
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
