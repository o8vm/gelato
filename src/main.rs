extern crate gelato;
extern crate irc;
extern crate futures;
extern crate tokio;

use gelato::view::App;
use iced::{Settings, Application};
use irc::client::prelude::*;
use futures::prelude::*;

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
  let config = Config::load("config.toml").unwrap();
  let mut client = Client::from_config(config).await?;
  client.identify()?;
  let mut stream = client.stream()?;
  let appcc = App::run(Settings::default());
  // transpose https://doc.rust-lang.org/std/result/enum.Result.html
  while let Some(message) = stream.next().await.transpose()? {
      print!("{}", message);
  }
  Ok(())
}