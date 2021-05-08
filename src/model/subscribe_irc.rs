use futures::*;
use iced_futures::futures;
use irc::client::{self, prelude::*};
use std::sync::{Arc};

// Subscriptionで登録されている関数。引数が渡せそう。
pub fn input<T: ToString>(
    client_stream: Option<&Arc<futures::lock::Mutex<irc::client::ClientStream>>>,
    some_input: T,
) -> iced::Subscription<Progress> {
    iced::Subscription::from_recipe(SubscribeIrc {
        client_stream: Some(Arc::clone(client_stream.unwrap())),
        some_input: some_input.to_string(),
    })
}

// だいぶ適当な実装にしてしまった。なんかよくわからないが回っているけれども、もう少しよく考えた方がいい。
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

    // 引数にBoxStreamがあって、返り値も同様
    fn stream(
        self: Box<Self>,
        _input: futures::stream::BoxStream<'static, I>,
    ) -> futures::stream::BoxStream<'static, Self::Output> {

        Box::pin(futures::stream::unfold(
            SubscribeIrcState::Ready {
                  client_stream: self.client_stream,
                  some_input: self.some_input,
            },
            |state| async move {
                match state {
                    // 最初に呼ばれるところ
                    SubscribeIrcState::Ready { client_stream, .. } => 
                    {
                      let mut client_stream = client_stream;
                      Some((
                            Progress::Started,
                            SubscribeIrcState::Incoming {
                                client_stream: client_stream,
                                message_text: String::from(""),
                            }
                      ))
                    },
                    // Streamが来ている時。回り続ける。
                    SubscribeIrcState::Incoming {
                        mut client_stream,
                        mut message_text,
                    } => {
                        let cloneclient = client_stream.clone();
                        match (&mut cloneclient.unwrap().lock().await).next().await.transpose() {
                            Ok(Some(chunk)) => {
                                message_text = chunk.to_string();
                                Some((
                                    Progress::Advanced(message_text.clone()),
                                    SubscribeIrcState::Incoming {
                                        client_stream,
                                        message_text,
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
        client_stream: Option<Arc<futures::lock::Mutex<irc::client::ClientStream>>>,
        some_input: String,
    },
    Incoming {
        client_stream: Option<Arc<futures::lock::Mutex<irc::client::ClientStream>>>,
        message_text: String,
    },
    Finished,
}

pub struct SubscribeIrc {
    some_input: String,
    client_stream: Option<Arc<futures::lock::Mutex<irc::client::ClientStream>>>,
}