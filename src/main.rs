extern crate gelato;
extern crate irc;
extern crate futures;
extern crate tokio;

use gelato::view::App;
use iced::{Settings, Application};

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
  App::run(Settings::default());
  Ok(())
}