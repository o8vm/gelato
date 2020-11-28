use iced::{Command};
use std::time::{Instant};
use crate::model::subscribe_irc;
use crate::app::*;
use crate::app::{SavedState, IrcError};
use futures::*;
use iced_futures::futures;
use std::sync::Arc;

// 読み込み済み、保存済み、入力変化した イベントの状態
#[derive(Debug, Clone)]
pub enum Message {
  Loaded(Result<SavedState, LoadError>),
  Saved(Result<(), SaveError>),
  InputChanged(String),
  IrcSet(State),
  SendText,
  Tick(Instant),
  IrcStart,
  IrcProgressed(subscribe_irc::Progress),
  IrcFinished(Result<(), IrcError>)
}

impl Message {
  pub async fn change() -> Result<(), IrcError> {
    Ok(())
  }
}

pub fn app_loading_command(app: &mut App, message: Message) -> Command<Message> {
  match message {
    Message::Loaded(Ok(state)) => {
      *app = App::Loaded(State::new_display_val(state.display_value));
    }
    Message::Loaded(Err(_)) => {
      *app = App::Loaded(State::default());
    }
    _ => {}
  }
  Command::none()
}