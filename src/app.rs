use iced::{
  button, text_input, Align, Application, Button, Column, Command, Container, Element, Length, scrollable, Scrollable, Settings, Subscription, Text, Row, Clipboard, TextInput
};
use iced_native::Rectangle;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

use crate::model::{subscribe_irc, subscribe_time, message::*};
use crate::view::util;


pub fn main() -> iced::Result {
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
  button2: button::State,
  scroll: scrollable::State,
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
      button2: button::State::new(),
      scroll: scrollable::State::new()
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
pub enum IrcError {
  IrcError
}

pub enum App {
  Loading,
  Loaded(State),
  IrcConnecting(State),
  IrcFinished(State),
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
  fn update(&mut self, message: Self::Message, _clipboard: &mut Clipboard) -> Command<Self::Message> {
    match self {
      App::Loading => app_loading_command(self, message),
      App::Loaded(state) => {
        let mut saved = false;
        let mut ircflag = false;
        match message {
          Message::Saved(_) => {
            state.saving = false;
            saved = true;
          }
          Message::IrcStart => {
            ircflag = true;
          }
          Message::Tick(now) => {
            let last_tick = &state.last_tick;
            state.duration += now - *last_tick;
            state.last_tick = now;
          }
          Message::InputChanged(value) => {
            state.input_value = value;
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
        } else if ircflag {
          let mut state_edit = state.clone();
          state_edit.progress = 0.0;
          *self = App::IrcConnecting(state_edit);
          Command::none()
        } else {
          Command::none()
        }
      }
      App::IrcConnecting(state) => {
        let mut irc_finished = false;
        match message {
          Message::IrcProgressed(dmessage) => match dmessage {
            subscribe_irc::Progress::Started => {
              state.progress = 0.0;
            }
            subscribe_irc::Progress::Advanced(message_text) => {
              let filtered_text: &str = util::filter(&message_text);
              state.display_value.push_str(filtered_text);
            }
            subscribe_irc::Progress::Finished => {
              irc_finished = true;
            }
            subscribe_irc::Progress::Errored => {
              irc_finished = true;
            }
          },
          Message::IrcFinished(_) => {
            irc_finished = true;
          }
          Message::InputChanged(value) => {
            state.input_value = value;
          }
          _ => {}
        }
        if irc_finished {
          *self = App::IrcFinished(state.clone());
          Command::perform(Message::change(), Message::IrcFinished)
        } else {
          Command::none()
        }
      }
      App::IrcFinished(state) => {
        *self = App::Loaded(state.clone());
        Command::none()
      }
    }
  }
  // サブスクリプションの登録
  fn subscription(&self) -> Subscription<Message> {
    match self {
      App::Loaded(State { .. })  => {
        subscribe_time::every(Duration::from_millis(10)).map(Message::Tick)
      }
      App::IrcConnecting (State{ .. }) => {
        subscribe_irc::input("")
          .map(Message::IrcProgressed)
      },
      _ => {
        Subscription::none()
      },
    }
  }
  // アプリケーションの表示を操作
  fn view(&mut self) -> Element<Self::Message> {
    match self {
      App::Loading => util::loading_message(),
      App::Loaded(state)
      | App::IrcFinished(state)
      | App::IrcConnecting(state) => {
        const MINUTE: u64 = 60;
        const HOUR: u64 = 60 * MINUTE;
        let seconds = state.duration.as_secs();
        let duration = Text::new(format!(
          "{:0>2}:{:0>2}:{:0>2}",
          seconds / HOUR,
          (seconds % HOUR) / MINUTE,
          seconds % MINUTE
        )).size(8);
        // Scrollable<scrollable::State> => Error
        let scrollable:Scrollable<Message> = Scrollable::new(&mut state.scroll)
            .width(Length::Fill)
            .height(Length::Fill)
            .push(Text::new(state.display_value.to_string()));
        //static b:button::State = *button;
        let control: Element<_> = {
          Button::new(&mut state.button, Text::new("Start IRC"))
            .on_press(Message::IrcStart)
            .into()
        };
        let control2: Element<_> = {
          Button::new(&mut state.button2, Text::new("Stop IRC"))
            .on_press(Message::IrcFinished(Ok(())))
            .into()
        };
        let input = TextInput::new(
          &mut state.input,
          "What needs to be done?",
          &mut state.input_value,
          Message::InputChanged,
        )
        .padding(15)
        .size(20)
        .on_submit(Message::CreateSendMessage);
        let content = Column::new()
          .padding(10)
          .spacing(10) 
          .align_items(Align::Start)
          //.push(duration)
          .push(control)
          .push(control2)
          .push(input)
          .push( Row::new()
          .align_items(Align::Center)
          .push(scrollable),);
        Container::new(content)
          .width(Length::FillPortion(2))
          .height(Length::Fill)
          .into()
      }
    }
  }
}
