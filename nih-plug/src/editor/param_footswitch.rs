use nih_plug::params::Param;
use vizia_plug::{vizia::prelude::*, widgets::param_base::ParamWidgetBase};

enum ParamFootswitchEvent {
  Pressed,
  Toggle,
  MomentaryTrigger,
  Reset,
}

pub struct ParamFootswitch {
  param_base: ParamWidgetBase,
  is_momentary: bool,
}

impl ParamFootswitch {
  pub fn new<F, L, Params, P, FMap>(
    cx: &mut Context,
    label_content: F,
    params: L,
    params_to_param: FMap,
  ) -> Handle<'_, Self>
  where
    L: Lens<Target = Params> + Clone,
    Params: 'static,
    P: Param<Plain = bool> + 'static,
    FMap: Fn(&Params) -> &P + Copy + 'static,
    F: FnOnce(&mut Context),
  {
    Self {
      param_base: ParamWidgetBase::new(cx, params, params_to_param),
      is_momentary: false,
    }
    .build(
      cx,
      ParamWidgetBase::build_view(params, params_to_param, |cx, param_data| {
        let value = param_data.make_lens(|param| param.modulated_plain_value());

        VStack::new(cx, |cx| {
          label_content(cx);
          HStack::new(cx, |cx| {
            ZStack::new(cx, |cx| {
              Element::new(cx)
                .shadow(Shadow::new(
                  0.,
                  0.,
                  Some(Length::px(8.0)),
                  Some(Length::px(4.0)),
                  Some(Color::RGBA(RGBA {
                    red: 184,
                    green: 0,
                    blue: 0,
                    alpha: 100,
                  })),
                  false,
                ))
                .class("footswitch-beam")
                .toggle_class("active", value);
            })
            .shadow(Shadow::new(
              0.,
              4.,
              Some(Length::px(4.0)),
              None,
              Some(Color::RGBA(RGBA {
                red: 0,
                green: 0,
                blue: 0,
                alpha: 25,
              })),
              true,
            ))
            .class("footswitch-light")
            .toggle_class("active", value);

            ZStack::new(cx, |cx| {
              Element::new(cx).class("footswitch-bg").hoverable(true);
              Element::new(cx)
                .class("footswitch-shadow")
                .toggle_class("active", value);
              Element::new(cx)
                .class("footswitch-fg")
                .toggle_class("active", value)
                .on_press(|cx| cx.emit(ParamFootswitchEvent::Pressed));
            })
            .vertical_gap(Pixels(8.0))
            .class("footswitch");
          })
          .size(Auto);
        })
        .size(Auto)
        .alignment(Alignment::Center)
        .vertical_gap(Pixels(3.0));
      }),
    )
  }
}

impl View for ParamFootswitch {
  fn element(&self) -> Option<&'static str> {
    Some("param-footswitch")
  }

  fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
    event.map(|param_event, meta| match param_event {
      ParamFootswitchEvent::Pressed => {
        if self.is_momentary {
          cx.emit(ParamFootswitchEvent::MomentaryTrigger);
        } else {
          cx.emit(ParamFootswitchEvent::Toggle);
        }
      }

      ParamFootswitchEvent::Toggle => {
        let normalized_value = self.param_base.modulated_normalized_value();
        self.param_base.begin_set_parameter(cx);
        self
          .param_base
          .set_normalized_value(cx, 1. - normalized_value);
        self.param_base.end_set_parameter(cx);

        meta.consume();
      }

      ParamFootswitchEvent::MomentaryTrigger => {
        self.param_base.begin_set_parameter(cx);
        self.param_base.set_normalized_value(cx, 1.0);
        self.param_base.end_set_parameter(cx);

        let entity = cx.current();
        cx.spawn(move |cx_proxy| {
          std::thread::sleep(std::time::Duration::from_millis(200));
          cx_proxy.emit_to(entity, ParamFootswitchEvent::Reset).ok();
        });

        meta.consume();
      }

      ParamFootswitchEvent::Reset => {
        self.param_base.begin_set_parameter(cx);
        self.param_base.set_normalized_value(cx, 0.0);
        self.param_base.end_set_parameter(cx);

        meta.consume();
      }
    });
  }
}

pub trait ParamFootswitchHandle {
  fn is_momentary(self, flag: bool) -> Self;
}

impl<'a> ParamFootswitchHandle for Handle<'a, ParamFootswitch> {
  fn is_momentary(self, flag: bool) -> Self {
    self.modify(|footswitch| {
      footswitch.is_momentary = flag;
    })
  }
}
