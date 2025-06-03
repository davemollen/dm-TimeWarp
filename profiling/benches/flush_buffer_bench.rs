#[path = "../src/utils.rs"]
mod utils;
use criterion::{criterion_group, criterion_main, Criterion};
use time_warp::StereoDelayLine;
use utils::generate_stereo_signal_stream;

fn flush_buffer(c: &mut Criterion) {
  let mut delay_line = StereoDelayLine::new(44100 * 10, 44100.);

  c.bench_function("flush_buffer", |b| {
    b.iter(|| {
      for _ in 0..200 {
        delay_line.reset();
      }
    })
  });
}

criterion_group!(benches, flush_buffer);
criterion_main!(benches);
