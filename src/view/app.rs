extern crate futures;
extern crate tokio;

use super::download;
use super::time;
use super::util;
use iced::{
  button,
  text_input,
  Align,
  Application,
  Button,
  Column,
  Command,
  Container,
  Element,
  Length,
  Settings,
  Subscription,
  Text, //TextInput,PaneGrid, pane_grid, Background, container,
};
use serde::{Deserialize, Serialize};
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
  progress: f32,
  button: button::State,
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
      progress: 0.0,
      button: button::State::new(),
    }
  }
}

// 読み込み済み、保存済み、入力変化した イベントの状態
#[derive(Debug, Clone)]
pub enum Message {
  Loaded(Result<SavedState, LoadError>),
  Saved(Result<(), SaveError>),
  InputChanged(String),
  Tick(Instant),
  Download,
  DownloadProgressed(download::Progress),
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
            *self = App::Loaded(State {
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
          Message::Tick(now) => {
            let last_tick = &state.last_tick;
            state.duration += now - *last_tick;
            state.last_tick = now;
          }
          Message::Download => match self {
            _ => {}
          },
          Message::DownloadProgressed(message) => match message {
            download::Progress::Started => {
              state.progress = 0.0;
            }
            download::Progress::Advanced(percentage) => {
              state.progress = percentage;
            }
            download::Progress::Finished => {}
            download::Progress::Errored => {}
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
              display_value: state.display_value.clone(),
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
  // サブスクリプションの登録
  fn subscription(&self) -> Subscription<Message> {
    match self {
      App::Loaded(State { display_value, .. }) => {
        time::every(Duration::from_millis(10)).map(Message::Tick)
        //Subscription::none()
      }
      _ => Subscription::none(),
    }
  }
  // アプリケーションの表示を操作
  fn view(&mut self) -> Element<Self::Message> {
    match self {
      App::Loading => util::loading_message(),
      App::Loaded(State {
        display_value,
        duration,
        button,
        ..
      }) => {
        const MINUTE: u64 = 60;
        const HOUR: u64 = 60 * MINUTE;
        let seconds = duration.as_secs();
        let duration = Text::new(format!(
          "{:0>2}:{:0>2}:{:0>2}",
          seconds / HOUR,
          (seconds % HOUR) / MINUTE,
          seconds % MINUTE
        ));
        //static b:button::State = *button;
        let control: Element<_> = {
          Button::new(button, Text::new("Start the download!"))
            .on_press(Message::Download)
            .into()
        };
        let content = Column::new()
          .padding(20)
          .spacing(20)
          .max_width(500)
          .align_items(Align::Start)
          .push(Text::new("test:"))
          .push(duration)
          .push(Text::new(display_value.to_string()))
          .push(control);
        Container::new(content)
          .width(Length::FillPortion(2))
          .height(Length::Fill)
          .into()
      }
    }
  }
}
