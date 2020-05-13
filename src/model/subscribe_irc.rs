use futures::*;
use iced_futures::futures;
use irc::client::prelude::*;

// Just a little utility function
pub fn input<T: ToString>(some_input: T) -> iced::Subscription<Progress> {
  iced::Subscription::from_recipe(SubscribeIrc {
    some_input: some_input.to_string(),
  })
}

pub struct SubscribeIrc {
  some_input: String,
}

async fn ircfunc() -> Result<irc::client::ClientStream, failure::Error> {
  let config = Config::load("config.toml").unwrap();
  let mut client = Client::from_config(config).await?;
  client.identify()?;
  // https://doc.rust-lang.org/std/option/enum.Option.html#method.transpose
  // transpose https://doc.rust-lang.org/std/result/enum.Result.html
  /*
  while let Some(message) = stream.next().await.transpose()? {
    print!("{}", message);
    };
  */
  Ok(client.stream()?)
}

// Make sure iced can use our download stream
impl<H, I> iced_native::subscription::Recipe<H, I> for SubscribeIrc
where
  H: std::hash::Hasher,
{
  type Output = Progress;

  fn hash(&self, state: &mut H) {
    use std::hash::Hash;

    std::any::TypeId::of::<Self>().hash(state);
    self.some_input.hash(state);
  }

  fn stream(
    self: Box<Self>,
    _input: futures::stream::BoxStream<'static, I>,
  ) -> futures::stream::BoxStream<'static, Self::Output> {
    Box::pin(futures::stream::unfold(
      SubscribeIrcState::Ready(self.some_input),
      |state| async move {
        match state {
          SubscribeIrcState::Ready(some_input) => {
            let result_stream: Result<irc::client::ClientStream, failure::Error> = ircfunc().await;
            match result_stream {
              Ok(client_stream) => {
                  Some((
                    Progress::Started,
                    SubscribeIrcState::Incoming {
                      client_stream,
                      message_text: String::from(""),
                    }
                  ))
              }
              Err(_) => {
                Some((Progress::Errored, SubscribeIrcState::Finished))
              },
            }
          }
          SubscribeIrcState::Incoming {
            mut client_stream,
            mut message_text,
          } => match client_stream.next().await.transpose() {
            Ok(Some(chunk)) => {
              //let downloaded = downloaded + chunk.to_string().len() as u64;
              //let percentage = (downloaded as f32 / total as f32) * 100.0;
              message_text.push_str(&chunk.to_string());
              Some((
                Progress::Advanced(message_text.clone()),
                SubscribeIrcState::Incoming {
                  client_stream,
                  message_text
                },
              ))
            }
            Ok(None) => Some((Progress::Finished, SubscribeIrcState::Finished)),
            Err(_) => Some((Progress::Errored, SubscribeIrcState::Finished)),
          },
          SubscribeIrcState::Finished => {
            // We do not let the stream die, as it would start a
            // new download repeatedly if the user is not careful
            // in case of errors.
            let _: () = iced::futures::future::pending().await;

            None
          }
        }
      },
    ))
  }
}

#[derive(Debug, Clone)]
pub enum Progress {
  Started,
  Advanced(String),
  Finished,
  Errored,
}

pub enum SubscribeIrcState {
  Ready(String),
  Incoming {
    client_stream: irc::client::ClientStream,
    message_text: String,
  },
  Finished,
}
