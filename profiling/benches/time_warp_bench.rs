#[path = "../src/utils.rs"]
mod utils;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use time_warp::{DelayLine, Interpolation, Notes, Params, TimeWarp};
use utils::generate_stereo_signal_stream;

// Benchmark for DelayLine::cosine_interp
//
// cosine_interp is private can be called via DelayLine::read with
// Interpolation::Cosine. The time value (5.33 ms) is chosen to produce a
// fractional sample offset (235.053 samples at 44 100 Hz), ensuring the cos()
// branch is always taken and never optimised away.
fn cosine_interp_bench(c: &mut Criterion) {
  const SAMPLE_RATE: f32 = 44100.;

  let mut delay_line = DelayLine::new((SAMPLE_RATE * 2.) as usize, SAMPLE_RATE);

  // Fill with a sine wave for some realistic waveform values
  for i in 0..(SAMPLE_RATE as usize * 2) {
    delay_line.write((i as f32 * std::f32::consts::TAU / 440.).sin());
  }

  let time = 5.33_f32;

  c.bench_function("cosine_interp", |b| {
    b.iter(|| black_box(delay_line.read(black_box(time), Interpolation::Cosine)))
  });
}

fn time_warp_bench(c: &mut Criterion) {
  let mut time_warp = TimeWarp::new(44100.);
  let mut params = Params::new(44100.);
  let mut notes = Notes::new();
  params.set(
    0.,
    0.1,
    false,
    1.,
    0.75,
    6.25,
    0.,
    0.,
    1.,
    true,
    true,
    time_warp::SampleMode::Delay,
    250.,
    1.,
    0.75,
    0.5,
    0.,
    0.,
    false,
    true,
    10.,
    50.,
    -12.,
    1000.,
    false,
    512,
  );
  time_warp.get_filter().set_cutoff_frequencies(200., 3000.);
  let signal_stream = generate_stereo_signal_stream(44100);

  c.bench_function("time_warp", |b| {
    b.iter(|| {
      for signal in &signal_stream {
        time_warp.process(*signal, &mut params, &mut notes.get_notes());
      }
    })
  });
}

criterion_group!(benches, time_warp_bench, cosine_interp_bench);
criterion_main!(benches);
