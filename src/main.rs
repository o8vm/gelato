extern crate gelato;

use gelato::view::App;
use iced::{Settings, Application};

extern crate futures;
extern crate irc;
extern crate tokio;

use irc::client::prelude::*;
use futures::prelude::*;

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
  let config = Config::load("config.toml").unwrap();
    let mut client = Client::from_config(config).await?;
    client.identify()?;

    let mut stream = client.stream()?;

    while let Some(message) = stream.next().await.transpose()? {
        print!("{}", message);
    }
    let app = App::run(Settings::default());
    Ok(())
}