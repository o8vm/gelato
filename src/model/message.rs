use iced::{Command};
use std::time::{Instant};
use crate::model::subscribe_irc;
use crate::app::*;
use crate::app::{SavedState, DownloadError};

// 読み込み済み、保存済み、入力変化した イベントの状態
#[derive(Debug, Clone)]
pub enum Message {
  Loaded(Result<SavedState, LoadError>),
  Saved(Result<(), SaveError>),
  InputChanged(String),
  Tick(Instant),
  Download,
  DownloadProgressed(subscribe_irc::Progress),
  Downloaded(Result<(), DownloadError>)
}

impl Message {
  pub async fn change() -> Result<(), DownloadError> {
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