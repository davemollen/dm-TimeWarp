mod utils;
use time_warp::{Notes, Params, TimeWarp};
use utils::generate_signal;

fn main() {
  let mut time_warp = TimeWarp::new(44100.);
  let mut params = Params::new(44100.);
  let mut notes = Notes::new();
  params.set(
    0.,
    0.1,
    0.75,
    1.,
    0.75,
    1.,
    true,
    true,
    time_warp::RecordMode::Delay,
    250.,
    1.,
    200.,
    3000.,
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
    time_warp.get_delay_line(),
    512,
  );

  loop {
    let input = (generate_signal(), generate_signal());
    time_warp.process(input, &mut params, &mut notes.get_notes());
  }
}
