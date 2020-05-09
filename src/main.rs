extern crate gelato;
extern crate irc;
extern crate futures;
extern crate tokio;

use gelato::view::App;
use iced::{Settings, Application};
use irc::client::prelude::*;
use futures::prelude::*;


async fn ircfunc() -> Result<(), failure::Error>{
  let config = Config::load("config.toml").unwrap();
  let mut client = Client::from_config(config).await?;
  client.identify()?;
  let mut stream = client.stream()?;
  // https://doc.rust-lang.org/std/option/enum.Option.html#method.transpose
  // transpose https://doc.rust-lang.org/std/result/enum.Result.html
  /*
  while let Some(message) = stream.next().await.transpose()? {
    print!("{}", message);
    };
  */
  while let Some(message) = stream.next().await {
    print!("{}", message?);
    };
  Ok(())
}

/*
async fn makeApp() -> Result<(), failure::Error> {
  let f1 = ircfunc();
  let f2 = ircfunc();
  futures::join!(f1, f2)
}
*/

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
  App::run(Settings::default());
  Ok(())
}