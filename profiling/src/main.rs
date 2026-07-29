mod utils;
use time_warp::{Notes, Params, TimeWarp};
use utils::generate_signal;

fn main() {
  let mut time_warp = TimeWarp::new(44100.);
  let mut params = Params::new(44100.);
  let mut notes = Notes::new();
  params.set(
    true,
    true,
    false,
    0.,
    0.1,
    false,
    1.,
    0.75,
    0.75,
    0.,
    0.,
    1.,
    time_warp::SampleMode::Delay,
    250.,
    1.,
    0.75,
    0.5,
    10.,
    50.,
    -12.,
    1000.,
    false,
    true,
    0.,
    0.,
    512,
  );
  time_warp.get_filter().set_cutoff_frequencies(200., 3000.);

  loop {
    let input = (generate_signal(), generate_signal());
    time_warp.process(input, &mut params, &mut notes.get_notes());
  }
}
