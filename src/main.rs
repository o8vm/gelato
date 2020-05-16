extern crate gelato;
extern crate irc;
extern crate futures;
extern crate tokio;


use iced::{Settings, Application};
#[tokio::main]
async fn main() -> Result<(), failure::Error> {
  let mut sets = Settings::default();
  sets.default_font = Some(include_bytes!("../fonts/M_PLUS_Rounded_1c/MPLUSRounded1c-Medium.ttf"));
  gelato::App::run(sets);
  Ok(())
}