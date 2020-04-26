extern crate futures;
extern crate tokio;

use iced::{
  text_input, Align, Application, Column, Command, Container, Subscription,
  Element, Length, Settings, Text, //TextInput,PaneGrid, pane_grid, Background, container,
};
use serde::{Deserialize, Serialize};
use super::util;

use std::time::{Duration, Instant};


// 本当は要らない
pub fn main() {
  App::run(Settings::default())
}

// アプリケーションの状態管理
pub struct State {
  input: text_input::State,
  input_value: String,
  display_value: String,
  saving: bool,
  dirty: bool,
  duration: Duration,
  last_tick: Instant,
}

impl Default for State {
  fn default() -> Self {
    Self {
      input: text_input::State::new(),
      input_value: String::from(""),
      display_value: String::from(""),
      saving: true,
      dirty: true,
      duration: Duration::default(),
      last_tick: std::time::Instant::now(),
    }
  }
}

// 読み込み済み、保存済み、入力変化した
#[derive(Debug, Clone)]
pub enum Message {
    Loaded(Result<SavedState, LoadError>),
    Saved(Result<(), SaveError>),
    InputChanged(String),
    Tick(Instant),
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
          },
          Message::Tick(now) => {
            let last_tick = &state.last_tick;
            state.duration += now - *last_tick;
            state.last_tick = now;
          },
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
            }.save(),
            Message::Saved,
          )
        } else {
          Command::none()
        }
      }
    }
  }
  // サブスクリプションの登録
  fn subscription(&self) -> Subscription<Message> {
    match self {
      App::Loaded(State {display_value, ..}) => {
        time::every(Duration::from_millis(10)).map(Message::Tick)
        //Subscription::none()
      },
      _ => {
        Subscription::none()
      }
  }
}
  // アプリケーションの表示を操作
   fn view(&mut self) -> Element<Self::Message> {
    match self {
      App::Loading => util::loading_message(),
      App::Loaded(State{
        display_value,
        duration,
        ..
      }) => util::show_display_val(&display_value, &duration),
    }
  }
}

// https://github.com/hecrj/iced/tree/master/examples/stopwatch
// stopwatchを移植してみた
mod time {
  use iced::futures;
  extern crate irc;
  use irc::client::prelude::*;
  use irc::client::ClientStream;

  pub async fn client_setting () -> Result<ClientStream, failure::Error> {
    let config = Config::load("config.toml").unwrap();
    let mut client = Client::from_config(config).await?;
    client.identify()?;
    let mut stream = client.stream()?;
    Ok(stream)
  }

  pub fn every(
      duration: std::time::Duration,
  ) -> iced::Subscription<std::time::Instant> {
      iced::Subscription::from_recipe(Every(duration))
  }

  struct Every(std::time::Duration);

  impl<H, I> iced_native::subscription::Recipe<H, I> for Every
  where
      H: std::hash::Hasher,
  {
      type Output = std::time::Instant;
      fn hash(&self, state: &mut H) {
          use std::hash::Hash;

          std::any::TypeId::of::<Self>().hash(state);
          self.0.hash(state);
      }

      fn stream(
          self: Box<Self>,
          _input: futures::stream::BoxStream<'static, I>,
      ) -> futures::stream::BoxStream<'static, Self::Output> {
          use futures::stream::StreamExt;
          let cstream = client_setting();
          // あと少し?
          async_std::stream::interval(self.0)
              .map(|_| std::time::Instant::now())
              .boxed()
      }
  }
}

