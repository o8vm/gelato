extern crate futures;
extern crate irc;
extern crate tokio;

use irc::client::prelude::*;
use futures::prelude::*;

pub struct Connection {
  config: Config,
  client: Client,
}

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    let config = Config::load("config.toml").unwrap();
    //let config = Config {
    //    nickname: Some("nick".to_owned()),
    //    server: Some("serverurl".to_owned()),
    //    channels: vec!["#channelname".to_owned()],
    //    use_ssl: true,
    //    ..Config::default()
    //};

    let mut client = Client::from_config(config).await?;
    client.identify()?;

    let mut stream = client.stream()?;

    while let Some(message) = stream.next().await.transpose()? {
        print!("{}", message);
    }
    Ok(())
}
