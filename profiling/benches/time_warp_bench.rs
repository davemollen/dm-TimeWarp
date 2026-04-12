#[path = "../src/utils.rs"]
mod utils;
use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use time_warp::{DelayLine, Interpolation, Notes, Params, TimeWarp};
use utils::generate_stereo_signal_stream;

// Benchmark targeting ADSR::apply_curve specifically.
//
// apply_curve is private, it is driven indirectly through TimeWarp::process
// with midi_enabled = true and a note held in the Attack stage.
//
// iter_batched supplies Notes + note_on each iteration so the ADSR
// always starts at x=0 in Attack; apply_curve is called on every
// sample. Attack is set to 5000 ms (≈960 000 samples at 192 000 Hz), which is
// far longer than SAMPLES_PER_ITER (500 000), so the stage never advances.
fn adsr_apply_curve_bench(c: &mut Criterion) {
  const SAMPLE_RATE: f32 = 192000.;
  const SAMPLES_PER_ITER: usize = 500000;

  let mut time_warp = TimeWarp::new(SAMPLE_RATE);
  let mut params = Params::new(SAMPLE_RATE);

  params.set(
    0.,
    0.,
    false,
    1.,
    0.75,
    6.25,
    0.,
    0.,
    0.,
    true,
    true,
    time_warp::SampleMode::Delay,
    250.,
    1.,
    0.75,
    0.5,
    0.,
    0.,
    true, // midi_enabled — routes processing through ADSR::apply_curve
    true,
    5000., // attack: long enough to stay in apply_curve for SAMPLES_PER_ITER samples
    500.,
    0.75,
    500.,
    false,
    512,
  );
  time_warp.get_filter().set_cutoff_frequencies(200., 3000.);

  let signal = (0.5_f32, 0.5_f32);

  c.bench_function("adsr_apply_curve", |b| {
    b.iter_batched(
      || {
        let mut notes = Notes::new();
        notes.note_on(60, 1.0);
        notes
      },
      |mut notes| {
        for _ in 0..SAMPLES_PER_ITER {
          time_warp.process(signal, &mut params, notes.get_notes());
        }
      },
      BatchSize::SmallInput,
    )
  });
}

// Benchmark targeting DelayLine::cosine_interp specifically.
//
// cosine_interp is private but reachable via DelayLine::read with
// Interpolation::Cosine. The time value (5.33 ms) is chosen to produce a
// fractional sample offset (235.053 samples at 44 100 Hz), ensuring the cos()
// branch is always taken and never optimised away.
fn cosine_interp_bench(c: &mut Criterion) {
  const SAMPLE_RATE: f32 = 44100.;

  let mut delay_line = DelayLine::new((SAMPLE_RATE * 2.) as usize, SAMPLE_RATE);

  // Fill with a sine wave so interpolation works on realistic values
  for i in 0..(SAMPLE_RATE as usize * 2) {
    delay_line.write((i as f32 * std::f32::consts::TAU / 440.).sin());
  }

  // 5.33 ms → 235.053 samples: fractional offset guarantees cos() is always called
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

criterion_group!(benches, time_warp_bench, adsr_apply_curve_bench, cosine_interp_bench);
criterion_main!(benches);
