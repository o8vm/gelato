extern crate futures;
extern crate irc;
extern crate tokio;


use iced::{
  text_input, Align, Application, Column, Command, Container,
  Element, Font, Length, Row, Settings, Text, TextInput,PaneGrid, pane_grid, Background, Color, container,
};
use serde::{Deserialize, Serialize};


pub fn main() {
  Gelato::run(Settings::default())
}

// アプリケーションの状態管理
#[derive(Debug, Default)]
struct State {
  input: text_input::State,
  input_value: String,
  display_value: Vec<String>,
  saving: bool,
  panes: usize,
  panes_created: usize,
}

#[derive(Debug, Clone)]
enum Message {
    Loaded(Result<SavedState, LoadError>),
    Saved(Result<(), SaveError>),
    InputChanged(String),
}

// Persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SavedState {
    input_value: String,
    display_value: Vec<String>,
}

#[derive(Debug, Clone)]
enum LoadError {
    FileError,
    FormatError,
}

#[derive(Debug, Clone)]
enum SaveError {
    DirectoryError,
    FileError,
    WriteError,
    FormatError,
}

#[cfg(not(target_arch = "wasm32"))]
impl SavedState {
}

enum Gelato {
  Loading,
  Loaded(State),
}

impl Application for Gelato {
  type Executor = iced::executor::Default;
  type Message = ();
  type Flags = ();

  // MUST
  fn new(_flags: ()) -> (Gelato, Command<Self::Message>) {
    (
      Gelato::Loading,
      Command::none(),
    )
  }

  // MUST
  fn title(&self) -> String {
    String::from("Gelato")
  }

  // MUST
    fn update(&mut self, _message: Self::Message) -> Command<Self::Message> {
    Command::none()
  }
    // MUST
    fn view(&mut self) -> Element<Self::Message> {
        let content = Column::new()
            .padding(20)
            .spacing(20)
            .max_width(500)
            .align_items(Align::Start)
            .push(Text::new("lilybrevec:"))
            .push(Text::new("MessageMofumofu"));
        Container::new(content)
            .width(Length::FillPortion(2))
            .height(Length::Fill)
            .into()
  }
}
