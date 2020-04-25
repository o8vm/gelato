extern crate futures;
extern crate tokio;

use iced::{
  text_input, Align, Application, Column, Command, Container,
  Element, Length, Settings, Text, //TextInput,PaneGrid, pane_grid, Background, container,
};
use serde::{Deserialize, Serialize};
use super::util;

// 本当は要らない
pub fn main() {
  App::run(Settings::default())
}

// アプリケーションの状態管理
#[derive(Debug, Default)]
pub struct State {
  input: text_input::State,
  input_value: String,
  display_value: String,
  saving: bool,
  dirty: bool,
  panes: usize,
  panes_created: usize,
}

// 読み込み済み、保存済み、入力変化した
#[derive(Debug, Clone)]
pub enum Message {
    Loaded(Result<SavedState, LoadError>),
    Saved(Result<(), SaveError>),
    InputChanged(String),
}

// 状態の内、保存する情報のモデル
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedState {
    input_value: String,
    display_value: String,
}

#[cfg(not(target_arch = "wasm32"))]
impl SavedState {
  // ファイルから状態を読み込む
  async fn load() -> Result<SavedState, LoadError> {
    let contents = r#"
        {
            "display_value": "Test for dispplay_value init",
            "input_value": "43"
        }"#;
    serde_json::from_str(&contents).map_err(|_| LoadError::FormatError)
}
  // ファイルに状態を保存
  async fn save(self) -> Result<(), SaveError> {
    Ok(())
  }
}


#[derive(Debug, Clone)]
pub enum LoadError {
    // ファイル読み込み時エラー状態名
    FileError,
    FormatError,
}

#[derive(Debug, Clone)]
pub enum SaveError {
    // 設定ファイル保存時のエラー状態名
    DirectoryError,
    FileError,
    WriteError,
    FormatError,
}

pub enum App {
  Loading,
  Loaded(State),
}

impl Application for App {
  type Executor = iced::executor::Default;
  type Message = Message;
  type Flags = ();

  // アプリケーションの初期化
  fn new(_flags: ()) -> (App, Command<Self::Message>) {
    (
      App::Loading,
      Command::perform(SavedState::load(), Message::Loaded),
    )
  }

  // アプリケーションのタイトル
  fn title(&self) -> String {
    String::from("Gelato")
  }

  // アプリケーションの更新
  fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
    match self {
      App::Loading => {
        match message {
          Message::Loaded(Ok(state)) => {
            *self = App::Loaded (State {
              display_value: state.display_value,
              ..State::default()
              });
          }
          Message::Loaded(Err(_)) => {
            *self = App::Loaded(State::default());
          }
          _ => {}
        }
        Command::none()
      }
      App::Loaded(state) => {
        let mut saved = false;
        match message {
          Message::Saved(_) => {
            state.saving = false;
            saved = true;
          }
          _ => {}
      }
      if !saved {
        state.dirty = true;
      }

    if state.dirty && !state.saving {
        state.dirty = false;
        state.saving = true;
        Command::perform(
          SavedState {
              input_value: state.input_value.clone(),
              display_value: state.display_value.clone()
          }
          .save(),
          Message::Saved,
        )
      } else {
        Command::none()
      }
    }
  }
}
  // アプリケーションの表示を操作
   fn view(&mut self) -> Element<Self::Message> {
    match self {
      App::Loading => util::loading_message(),
      App::Loaded(State {
        display_value, ..
      }) => util::show_display_val(&display_value),
    }
  }
}