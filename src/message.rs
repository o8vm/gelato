use crate::app::*;
use crate::app::{IrcError, SavedState};
use crate::subscribe_irc;
use anyhow::Result;
use iced::Command;

// 読み込み済み、保存済み、入力変化した イベントの状態
// TODO: クライアントの作成時awaitの非同期処理が走る。
// ResultがMessageに渡されるが、MessageはCloneする必要がある。
// irc::error::Error、継承元のio::Error, failure::Error, またanyhowいずれもclone実装がない
// Box化するのが定番のようだが、Box化してしまうと.await?が使えないように感じている。

#[derive(Debug, Clone)]
pub enum Message {
    Loaded(Result<SavedState, LoadError>),
    Saved(Result<(), SaveError>),
    InputChanged(String),
    IrcStart,
    IrcProgressed(subscribe_irc::Progress),
    IrcFinished(Result<(), IrcError>),
    PostMessage,
    None(()),
}

impl Message {
    pub async fn change() -> Result<(), IrcError> {
        Ok(())
    }
}

pub fn app_loading_command(app: &mut App, message: Message) -> Command<Message> {
    match message {
        // saved_stateであることに注意すること。
        Message::Loaded(Ok(_saved_state)) => {
            *app = App::Loaded(State::new_display_val(_saved_state.display_value));
        }
        Message::Loaded(Err(_)) => {
            *app = App::Loaded(State::default());
        }
        _ => {}
    }
    Command::none()
}
