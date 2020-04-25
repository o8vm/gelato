extern crate futures;
extern crate tokio;

use iced::{
  text_input, Align, Application, Column, Command, Container,
  Element, Length, Settings, Text, //TextInput,PaneGrid, pane_grid, Background, container,
};
use serde::{Deserialize, Serialize};


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
  panes: usize,
  panes_created: usize,
}

#[derive(Debug, Clone)]
pub enum Message {
    Loaded(Result<SavedState, LoadError>),
    Saved(Result<(), SaveError>),
    InputChanged(String),
}

// Persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedState {
    input_value: String,
    display_value: String,
}

#[derive(Debug, Clone)]
pub enum LoadError {
    FileError,
    FormatError,
}

#[derive(Debug, Clone)]
pub enum SaveError {
    DirectoryError,
    FileError,
    WriteError,
    FormatError,
}

#[cfg(not(target_arch = "wasm32"))]
impl SavedState {
}

pub enum App {
  Loading,
  Loaded(State),
}

impl Application for App {
  type Executor = iced::executor::Default;
  type Message = Message;
  type Flags = ();

  // MUST
  fn new(_flags: ()) -> (App, Command<Self::Message>) {
    (
      App::Loading,
      Command::none(),
    )
  }

  // MUST
  fn title(&self) -> String {
    String::from("Gelato")
  }

  // MUST
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
      Command::none()
    }
  }
}
  // MUST
   fn view(&mut self) -> Element<Self::Message> {
    match self {
      App::Loading => loading_message(),
      App::Loaded(State {
        display_value, ..
      }) => {
        let content = Column::new()
            .padding(20)
            .spacing(20)
            .max_width(500)
            .align_items(Align::Start)
            .push(Text::new("lilybrevec:"))
            .push(Text::new(display_value.to_string()));
        Container::new(content)
            .width(Length::FillPortion(2))
            .height(Length::Fill)
            .into()
      }
  }
}
}


fn loading_message() -> Element<'static, Message> {
  Container::new(
      Text::new("Loading...")
          .size(50),
  )
  .width(Length::Fill)
  .height(Length::Fill)
  .center_y()
  .into()
}