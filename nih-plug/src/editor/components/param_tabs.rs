use nih_plug::params::Param;
use vizia_plug::{vizia::prelude::*, widgets::param_base::ParamWidgetBase};

enum ParamTabEvent {
  TextInput(String),
}

pub struct ParamTabs {
  param_base: ParamWidgetBase,
}

impl ParamTabs {
  pub fn new<'a, L, Params, P, FMap>(
    cx: &'a mut Context,
    options: &'static [&'static str],
    params: L,
    params_to_param: FMap,
  ) -> Handle<'a, Self>
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
        VStack::new(cx, |cx| {
          Label::new(cx, param_data.param().name()).alignment(Alignment::Center);
          VStack::new(cx, |cx| {
            for option in options {
              let checked = param_data.make_lens(|param| param.to_string() == option.to_string());

              Label::new(cx, &option.to_uppercase())
                .checkable(true)
                .on_press(|cx| cx.emit(ParamTabEvent::TextInput(option.to_string())))
                .checked(checked)
                .alignment(Alignment::Center)
                .class("tab");
            }
          });
        })
        .alignment(Alignment::Center)
        .vertical_gap(Pixels(8.0));
      }),
    )
  }
}

impl View for ParamTabs {
  fn element(&self) -> Option<&'static str> {
    Some("param-tabs")
  }

  fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
    event.map(|param_event, meta| match param_event {
      ParamTabEvent::TextInput(string) => {
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
