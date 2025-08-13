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
    0.5,
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
  let derived_params = params.get_derived_params();

  loop {
    let input = (generate_signal(), generate_signal());
    time_warp.process(input, &mut params, &mut notes.get_notes(), &derived_params);
  }
}
