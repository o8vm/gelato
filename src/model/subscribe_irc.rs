use futures::*;
use iced_futures::futures;
use irc::client::prelude::*;

// Just a little utility function
pub fn input<T: ToString>(
    post_flag: bool,
    input_value: &str,
    some_input: T,
) -> iced::Subscription<Progress> {
    iced::Subscription::from_recipe(SubscribeIrc {
        post_flag: post_flag,
        input_value: input_value.to_string(),
        some_input: some_input.to_string(),
    })
}

async fn get_irc_client() -> Result<IrcClient, failure::Error> {
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
    Ok(IrcClient {
        client_stream: client.stream()?,
        sender: client.sender(),
    })
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
            SubscribeIrcState::Ready {
                post_flag: self.post_flag,
                input_value: self.input_value,
                some_input: self.some_input,
            },
            |state| async move {
                match state {
                    SubscribeIrcState::Ready { .. } => match get_irc_client().await {
                        Ok(IrcClient {
                            client_stream,
                            sender,
                        }) => Some((
                            Progress::Started,
                            SubscribeIrcState::Incoming {
                                client_stream,
                                sender,
                                message_text: String::from(""),
                                post_flag: false,
                                input_value: String::from(""),
                            },
                        )),
                        Err(_) => Some((Progress::Errored, SubscribeIrcState::Finished)),
                    },
                    SubscribeIrcState::Incoming {
                        mut client_stream,
                        sender,
                        mut message_text,
                        post_flag,
                        input_value,
                    } => {
                        match client_stream.next().await.transpose() {
                            Ok(Some(chunk)) => {
                              //sender.send_privmsg("#test", "test").unwrap();
                              //message_text.push_str("client test");
                                message_text = chunk.to_string();
                                Some((
                                    Progress::Advanced(message_text.clone()),
                                    SubscribeIrcState::Incoming {
                                        client_stream,
                                        sender,
                                        message_text,
                                        post_flag: false,
                                        input_value: String::from(""),
                                    },
                                ))
                            }
                            Ok(None) => Some((Progress::Finished, SubscribeIrcState::Finished)),
                            Err(_) => Some((Progress::Errored, SubscribeIrcState::Finished)),
                        }
                    }
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
    Ready {
        post_flag: bool,
        input_value: String,
        some_input: String,
    },
    Incoming {
        client_stream: irc::client::ClientStream,
        sender: irc::client::Sender,
        message_text: String,
        post_flag: bool,
        input_value: String,
    },
    Finished,
}

pub struct SubscribeIrc {
    post_flag: bool,
    some_input: String,
    input_value: String,
}

struct IrcClient {
    client_stream: irc::client::ClientStream,
    sender: irc::client::Sender,
}
