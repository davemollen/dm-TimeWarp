use nih_plug::prelude::AtomicF32;
use std::sync::{atomic::Ordering, Arc};
use time_warp::MIN_DELAY_TIME;

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
    let time_segment = string.trim().to_ascii_lowercase();

    if let Some(val) = time_segment.strip_suffix("ms") {
      val.trim().parse::<f32>().ok()
    } else if let Some(val) = time_segment.strip_suffix('s') {
      val.trim().parse::<f32>().ok().map(|x| x * 1000.0)
    } else {
      time_segment.parse::<f32>().ok()
    }
  })
}

pub fn v2s_f32_synced_time() -> Arc<dyn Fn(i32) -> String + Send + Sync> {
  Arc::new(move |value| {
    match value {
      0 => "1/32",
      1 => "1/16T",
      2 => "1/32.",
      3 => "1/16",
      4 => "1/8T",
      5 => "1/16.",
      6 => "1/8",
      7 => "1/4T",
      8 => "1/8.",
      9 => "1/4",
      10 => "1/2T",
      11 => "1/4.",
      12 => "1/2",
      13 => "1T",
      14 => "1/2.",
      15 => "1",
      16 => "2T",
      17 => "1.",
      18 => "2",
      19 => "4T",
      20 => "2.",
      _ => "",
    }
    .to_string()
  })
}

pub fn s2v_f32_synced_time() -> Arc<dyn Fn(&str) -> Option<i32> + Send + Sync> {
  Arc::new(|string| match string {
    "1/32" => Some(0),
    "1/16T" => Some(1),
    "1/32." => Some(2),
    "1/16" => Some(3),
    "1/8T" => Some(4),
    "1/16." => Some(5),
    "1/8" => Some(6),
    "1/4T" => Some(7),
    "1/8." => Some(8),
    "1/4" => Some(9),
    "1/2T" => Some(10),
    "1/4." => Some(11),
    "1/2" => Some(12),
    "1T" => Some(13),
    "1/2." => Some(14),
    "1" => Some(15),
    "2T" => Some(16),
    "1." => Some(17),
    "2" => Some(18),
    "4T" => Some(19),
    "2." => Some(20),
    _ => None,
  })
}

pub fn v2s_size(size_max: Arc<AtomicF32>) -> Arc<dyn Fn(f32) -> String + Send + Sync> {
  Arc::new(move |value| {
    let range = size_max.load(Ordering::Relaxed) - MIN_DELAY_TIME;
    let value = value * value * range + MIN_DELAY_TIME;
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

pub fn s2v_size(size_max: Arc<AtomicF32>) -> Arc<dyn Fn(&str) -> Option<f32> + Send + Sync> {
  Arc::new(move |string| {
    let time_segment = string.trim().to_ascii_lowercase();

    let plain_value = if let Some(val) = time_segment.strip_suffix("ms") {
      val.trim().parse::<f32>().ok()
    } else if let Some(val) = time_segment.strip_suffix('s') {
      val.trim().parse::<f32>().ok().map(|x| x * 1000.0)
    } else {
      time_segment.parse::<f32>().ok()
    };

    match plain_value {
      Some(plain_val) => {
        let max = size_max.load(Ordering::Relaxed);
        let range = max - MIN_DELAY_TIME;
        let val = if plain_val < MIN_DELAY_TIME {
          MIN_DELAY_TIME
        } else if plain_val > max {
          max
        } else {
          plain_val
        };
        let y = (val - MIN_DELAY_TIME) / range;
        Some(y.sqrt())
      }
      _ => None,
    }
  })
}
