use std::f32::consts::PI;

use nih_plug_vizia::vizia::{
  binding::{Lens, LensExt},
  context::{Context, DrawContext, EmitContext, EventContext},
  layout::Units,
  vg::{LineCap, Paint, Path, Solidity},
  view::{Canvas, Handle, View},
  views::{Binding, KnobMode},
};

enum ArcTrackEvent {
  SetValue(f32),
}

/// Makes a knob that represents the current value with an arc
pub struct ArcTrack {
  angle_start: f32,
  angle_end: f32,
  radius: Units,
  span: Units,
  normalized_value: f32,

  center: bool,
  mode: KnobMode,
}

impl ArcTrack {
  /// Creates a new [ArcTrack] view.
  pub fn new<L: Lens<Target = f32>>(
    cx: &mut Context,
    lens: L,
    center: bool,
    radius: Units,
    span: Units,
    angle_start: f32,
    angle_end: f32,
    mode: KnobMode,
  ) -> Handle<'_, Self> {
    Self {
      angle_start,
      angle_end,
      radius,
      span,
      normalized_value: 0.5,
      center,
      mode,
    }
    .build(cx, |cx| {
      Binding::new(cx, lens, |cx, val| {
        let val = val.get(cx);
        cx.emit(ArcTrackEvent::SetValue(val));
      });
    })
  }
}

impl View for ArcTrack {
  fn element(&self) -> Option<&'static str> {
    Some("arctrack")
  }

  fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
    let foreground_color = cx.font_color().into();

    let background_color = cx.background_color().into();

    let bounds = cx.bounds();

    // Calculate arc center
    let centerx = bounds.x + 0.5 * bounds.w;
    let centery = bounds.y + 0.5 * bounds.h;

    // Convert start and end angles to radians and rotate origin direction to be upwards instead of to the right
    let start = self.angle_start.to_radians() - PI / 2.0;
    let end = self.angle_end.to_radians() - PI / 2.0;

    let parent_width = bounds.w + cx.logical_to_physical(2.0);

    // Convert radius and span into screen coordinates
    let radius = self.radius.to_px(parent_width / 2.0, 0.0);
    // default value of span is 15 % of radius. Original span value was 16.667%
    let span = self.span.to_px(radius, 0.0);

    // Draw the track arc
    let mut path = Path::new();
    path.arc(
      centerx,
      centery,
      radius - span / 2.0,
      end,
      start,
      Solidity::Solid,
    );
    let mut paint = Paint::color(background_color);
    paint.set_line_width(span);
    paint.set_line_cap(LineCap::Round);
    canvas.stroke_path(&path, &paint);

    // Draw the active arc
    let mut path = Path::new();

    let value = match self.mode {
      KnobMode::Continuous => self.normalized_value,
      // snapping
      KnobMode::Discrete(steps) => {
        (self.normalized_value * (steps - 1) as f32).floor() / (steps - 1) as f32
      }
    };

    if self.center {
      let center = -PI / 2.0;

      if value <= 0.5 {
        let current = value * 2.0 * (center - start) + start;
        path.arc(
          centerx,
          centery,
          radius - span / 2.0,
          center,
          current,
          Solidity::Solid,
        );
      } else {
        let current = (value * 2.0 - 1.0) * (end - center) + center;
        path.arc(
          centerx,
          centery,
          radius - span / 2.0,
          current,
          center,
          Solidity::Solid,
        );
      }
    } else {
      let current = value * (end - start) + start;
      path.arc(
        centerx,
        centery,
        radius - span / 2.0,
        current,
        start,
        Solidity::Solid,
      );
    }

    let mut paint = Paint::color(foreground_color);
    paint.set_line_width(span);
    paint.set_line_cap(LineCap::Round);
    canvas.stroke_path(&path, &paint);
  }

  fn event(&mut self, cx: &mut EventContext, event: &mut nih_plug_vizia::vizia::events::Event) {
    event.map(|msg, _| match msg {
      ArcTrackEvent::SetValue(val) => {
        self.normalized_value = *val;
        cx.needs_redraw();
      }
    });
  }
}
