use futures::*;
use iced_futures::futures;
use irc::client::prelude::{Client, Config};
use std::sync::Arc;

// Just a little utility function
pub fn input(client: Arc<Option<irc::client::Client>>) -> iced::Subscription<Progress> {
  iced::Subscription::from_recipe(SubscribeIrc {
    some_input: "".to_string(),
    client: client
  })
}

pub struct SubscribeIrc {
  some_input: String,
  client: Arc<Option<irc::client::Client>>
}

async fn ircfunc(client: Arc<Option<irc::client::Client>>) -> Result<irc::client::ClientStream, failure::Error> {
  //client.send_privmsg("#mofu", "beepj").unwrap();
  let mut c = client.take();
  Ok(c.unwrap().stream()?)
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
          SubscribeIrcState::Ready(_) => {
            let mut c = self.client.clone();
            let result_stream: Result<irc::client::ClientStream, failure::Error> = ircfunc(c).await;
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
              message_text = chunk.to_string();
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
