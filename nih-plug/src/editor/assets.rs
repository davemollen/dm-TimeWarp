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
