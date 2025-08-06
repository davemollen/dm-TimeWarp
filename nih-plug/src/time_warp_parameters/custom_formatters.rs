use std::sync::Arc;

pub fn v2s_f32_ms_then_s() -> Arc<dyn Fn(f32) -> String + Send + Sync> {
  Arc::new(move |value| {
    if value >= 10000. {
      format!("{:.1} s", value / 1000.0)
    } else if value >= 1000. {
      format!("{:.2} s", value / 1000.0)
    } else if value >= 100. {
      format!("{value:.0} ms")
    } else if value >= 10. {
      format!("{value:.1} ms")
    } else {
      format!("{value:.2} ms")
    }
  })
}

pub fn s2v_f32_ms_then_s() -> Arc<dyn Fn(&str) -> Option<f32> + Send + Sync> {
  Arc::new(move |string| {
    let string = string.trim();
    let mut segments = string.split(',');
    let segments = (segments.next(), segments.next(), segments.next());

    let time_segment = segments.0?;
    let cleaned_string = time_segment.trim_end_matches([' ', 's', 'S']).parse().ok();
    match time_segment.get(time_segment.len().saturating_sub(3)..) {
      Some(unit) if unit.eq_ignore_ascii_case("s") => cleaned_string.map(|x| x * 1000.0),
      _ => cleaned_string,
    }
  })
}
