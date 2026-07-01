#[macro_export]
macro_rules! assert_approximately_eq {
  ($left:expr, $right:expr, $digits:expr $(,)?) => {{
    let tol = 10f32.powi(-($digits as i32));
    let diff = ($left - $right).abs();
    assert!(
      diff <= tol,
      "Values are not approximately equal: left={}, right={}, diff={}, tol={}",
      $left,
      $right,
      diff,
      tol
    );
  }};
}

#[macro_export]
macro_rules! assert_complex_num_approximately_eq {
  ($left:expr, $right:expr, $digits:expr $(,)?) => {{
    let tol = 10f32.powi(-($digits as i32));
    let re_diff = ($left.re - $right.re).abs();
    let im_diff = ($left.im - $right.im).abs();
    assert!(
      re_diff <= tol && im_diff <= tol,
      "Values are not approximately equal: left={}, right={}, re_diff={}, im_diff={}, tol={}",
      $left,
      $right,
      re_diff,
      im_diff,
      tol
    );
  }};
}
