extern crate gelato;
extern crate irc;
extern crate futures;
extern crate tokio;


use iced::{Settings, Application};
#[tokio::main]
async fn main() -> Result<(), failure::Error> {
  gelato::App::run(Settings::default());
  Ok(())
}