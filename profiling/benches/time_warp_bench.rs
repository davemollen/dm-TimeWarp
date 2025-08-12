#[path = "../src/utils.rs"]
mod utils;
use criterion::{criterion_group, criterion_main, Criterion};
use time_warp::{Notes, Params, TimeWarp};
use utils::generate_stereo_signal_stream;

fn time_warp_bench(c: &mut Criterion) {
  let mut time_warp = TimeWarp::new(44100.);
  let mut params = Params::new(44100.);
  let mut notes = Notes::new();
  params.set(
    0.,
    0.1,
    0.75,
    0.75,
    0.,
    1.,
    1.,
    true,
    true,
    time_warp::RecordMode::Delay,
    250.,
    1.,
    0.75,
    0.,
    0.,
    0.,
    false,
    10.,
    50.,
    -12.,
    1000.,
    false,
    512,
  );
  time_warp.get_filter().set_coefficients(200., 3000.);
  let signal_stream = generate_stereo_signal_stream(44100);

  c.bench_function("time_warp", |b| {
    b.iter(|| {
      for signal in &signal_stream {
        time_warp.process(*signal, &mut params, &mut notes.get_notes());
      }
    })
  });
}

criterion_group!(benches, time_warp_bench);
criterion_main!(benches);
