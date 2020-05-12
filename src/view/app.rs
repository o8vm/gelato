extern crate futures;
extern crate tokio;

use super::download;
use super::message::*;
use super::time;
use super::util;
use iced::{
  button, text_input, Align, Application, Button, Column, Command, Container, Element, Length,
  ProgressBar, Settings, Subscription, Text,
};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

// 本当は要らない
pub fn main() {
  App::run(Settings::default())
}

// アプリケーションの状態管理
#[derive(Debug, Clone)]
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

impl State {
  pub fn new_display_val(s: String) -> Self {
    let mut default: State = State::default();
    default.display_value = String::from(s.to_string());
    default
  }
  pub fn new_progress(v: f32) -> Self {
    let mut default: State = State::default();
    default.progress = v;
    default
  }
}
// 状態の内、保存する情報のモデル
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedState {
  pub input_value: String,
  pub display_value: String,
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
#[derive(Debug, Clone)]
pub enum DownloadError {
  DownloadError
}

pub enum App {
  Loading,
  Loaded(State),
  Downloading(State),
  Downloaded(State),
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
      App::Loading => app_loading_command(self, message),
      App::Loaded(state) => {
        let mut saved = false;
        let mut download = false;
        match message {
          Message::Saved(_) => {
            state.saving = false;
            saved = true;
          }
          Message::Download => {
            download = true;
          }
          Message::Tick(now) => {
            let last_tick = &state.last_tick;
            state.duration += now - *last_tick;
            state.last_tick = now;
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
              display_value: state.display_value.clone(),
            }
            .save(),
            Message::Saved,
          )
        } else if download {
          let mut state_edit = state.clone();
          state_edit.progress = 0.0;
          *self = App::Downloading(state_edit);
          Command::none()
        } else {
          Command::none()
        }
      }
      App::Downloading(state) => {
        let mut download_done = false;
        match message {
          Message::DownloadProgressed(dmessage) => match dmessage {
            download::Progress::Started => {
              state.progress = 0.0;
            }
            download::Progress::Advanced(message_text) => {
              state.progress = 0.0;
              state.display_value = message_text;
            }
            download::Progress::Finished => {
              download_done = true;
            }
            download::Progress::Errored => {
              download_done = true;
            }
          },
          _ => {}
        }
        if download_done {
          *self = App::Downloaded(state.clone());
          Command::perform(Message::change(), Message::Downloaded)
        } else {
          Command::none()
        }
      }
      App::Downloaded(state) => {
        *self = App::Loaded(state.clone());
        Command::none()
      }
    }
  }
  // サブスクリプションの登録
  fn subscription(&self) -> Subscription<Message> {
    match self {
      App::Loaded(State { .. })  => {
        time::every(Duration::from_millis(10)).map(Message::Tick)
      }
      App::Downloading (State{ .. }) => {
        download::file("https://speed.hetzner.de/100MB.bin")
          .map(Message::DownloadProgressed)
      },
      _ => {
        Subscription::none()
      },
    }
  }
  // アプリケーションの表示を操作
  fn view(&mut self) -> Element<Self::Message> {
    let current_progress = match self {
      App::Downloading(State { progress, .. })
      | App::Loaded(State { progress, ..}) => *progress,
      App::Downloaded { .. } => 100.0,
      _ => 0.0,
    };
    let progress_bar = ProgressBar::new(0.0..=100.0, current_progress);
    match self {
      App::Loading => util::loading_message(),
      App::Loaded(state)
      | App::Downloaded(state)
      | App::Downloading(state) => {
        const MINUTE: u64 = 60;
        const HOUR: u64 = 60 * MINUTE;
        let seconds = state.duration.as_secs();
        let duration = Text::new(format!(
          "{:0>2}:{:0>2}:{:0>2}",
          seconds / HOUR,
          (seconds % HOUR) / MINUTE,
          seconds % MINUTE
        ));
        //static b:button::State = *button;
        let control: Element<_> = {
          Button::new(&mut state.button, Text::new("Start the download!"))
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
          .push(Text::new(state.display_value.to_string()))
          .push(control)
          .push(progress_bar);
        Container::new(content)
          .width(Length::FillPortion(2))
          .height(Length::Fill)
          .into()
      }
    }
  }
}
