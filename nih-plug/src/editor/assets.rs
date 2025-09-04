use vizia_plug::vizia::prelude::Context;

// FONTS
pub const ROBOTO_FONT_NAME: &str = "Roboto";
pub const ROBOTO_REGULAR: &[u8] = include_bytes!("./assets/fonts/roboto/Roboto-Regular.ttf");
pub const ROBOTO_BOLD: &[u8] = include_bytes!("./assets/fonts/roboto/Roboto-Bold.ttf");

pub fn register_roboto(cx: &mut Context) {
  cx.add_font_mem(ROBOTO_REGULAR);
}
pub fn register_roboto_bold(cx: &mut Context) {
  cx.add_font_mem(ROBOTO_BOLD);
}

// SVG Icons
pub const RECORD_ICON: &str = r#"
<svg
  width="19"
  height="19"
  viewBox="0 0 19 19"
  fill="currentColor"
  xmlns="http://www.w3.org/2000/svg"
  class="record"
>
  <path
    d="M9.97727 18.2727C8.73864 18.2727 7.57386 18.0398 6.48295 17.5739C5.39773 17.108 4.44034 16.4631 3.6108 15.6392C2.78693 14.8097 2.1392 13.8523 1.66761 12.767C1.2017 11.6818 0.971591 10.517 0.977273 9.27273C0.982955 8.02841 1.21875 6.86364 1.68466 5.77841C2.15625 4.69318 2.80398 3.73864 3.62784 2.91477C4.4517 2.08523 5.40625 1.4375 6.49148 0.971591C7.5767 0.505682 8.73864 0.272727 9.97727 0.272727C11.2216 0.272727 12.3864 0.505682 13.4716 0.971591C14.5625 1.4375 15.517 2.08523 16.3352 2.91477C17.1591 3.73864 17.804 4.69318 18.2699 5.77841C18.7358 6.86364 18.9716 8.02841 18.9773 9.27273C18.983 10.517 18.7528 11.6818 18.2869 12.767C17.821 13.8523 17.1761 14.8097 16.3523 15.6392C15.5284 16.4631 14.571 17.108 13.4801 17.5739C12.3892 18.0398 11.2216 18.2727 9.97727 18.2727Z"
  />
</svg>
"#;

pub const PLAY_ICON: &str = r#"
<svg
  width="19"
  height="23"
  viewBox="0 0 19 23"
  fill="currentColor"
  xmlns="http://www.w3.org/2000/svg"
  class="play"
>
  <path d="M0.25 22.1818V0.363635L18.7955 11.2727L0.25 22.1818Z" />
</svg>
"#;

pub const STOP_ICON: &str = r#"
<svg
  width="18"
  height="18"
  viewBox="0 0 18 18"
  fill="currentColor"
  xmlns="http://www.w3.org/2000/svg"
  class="stop"
>
  <path d="M0.25 18V0.545454H17.7045V18H0.25Z" />
</svg>
"#;
