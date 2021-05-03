use iced::{
    button, scrollable, text_input, Align, Application, Button, Clipboard, Column, Command,
    Container, Element, Length, Row, Scrollable, Settings, Subscription, Text, TextInput,
};
use serde::{Deserialize, Serialize};
use std::{
    ptr::null,
    time::{Duration, Instant},
};

use crate::model::{message::*, subscribe_irc};
use crate::style;
use crate::util;
use irc::client::prelude::{Client, Config, Sender};

pub fn main() -> iced::Result {
    App::run(Settings::default())
}

// アプリケーションの状態管理
#[derive(Debug, Clone)]
pub struct State {
    input_state: text_input::State,
    input_value: String,
    connecting_flag: bool,
    display_value: String,
    saving: bool,
    dirty: bool,
    duration: Duration,
    last_tick: Instant,
    progress: f32,
    post_flag: bool,
    irc_button_state: button::State,
    post_button_state: button::State,
    scrollable_state: scrollable::State,
    sender: Option<Box<irc::client::Sender>>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            input_state: text_input::State::new(),
            input_value: String::from(""),
            connecting_flag: false,
            display_value: String::from(""),
            saving: true,
            dirty: true,
            duration: Duration::default(),
            last_tick: std::time::Instant::now(),
            progress: 0.0,
            post_flag: false,
            irc_button_state: button::State::new(),
            post_button_state: button::State::new(),
            scrollable_state: scrollable::State::new(),
            sender: None,
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
    pub display_value: String
}

struct IrcClient {
    client_stream: irc::client::ClientStream,
    sender: irc::client::Sender,
}
impl IrcClient {
    async fn get_sender() -> Result<irc::client::Sender, failure::Error> {
        let config = Config::load("config.toml").unwrap();
        let mut client = Client::from_config(config).await?;
        client.identify()?;
        Ok(client.sender())
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl SavedState {
    // ファイルから状態を読み込む
    async fn load() -> Result<SavedState, LoadError> {
        let contents = r#"
        {
            "display_value": "Push Start IRC button",
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
    IrcError,
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
    fn update(
        &mut self,
        message: Self::Message,
        _clipboard: &mut Clipboard,
    ) -> Command<Self::Message> {
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
                state.connecting_flag = true;
                let mut irc_finished = false;
                let mut posted = false;
                match message {
                    Message::IrcProgressed(progress_state) => match progress_state {
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
                        state.connecting_flag = false;
                    }
                    Message::InputChanged(value) => {
                        state.input_value = value;
                    }
                    Message::PostMessage => {
                      posted = true;
                    }
                    _ => {}
                }

                if posted {
                  let call = async move {
                    match IrcClient::get_sender().await {
                      Ok(sender) => {
                        sender.send_privmsg("#test", "momomomo").unwrap();
                      }
                      Err(msg) => println!("failure: {}", msg)
                    }
                    //let config = Config::load("config.toml").unwrap();
                    //let mut client = Client::from_config(config).await.unwrap();
                    //client.identify().unwrap();
                  };
                  Command::perform(call, Message::None)
                } else if irc_finished {
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
            /*App::Loaded(State { .. })  => {
              subscribe_time::every(Duration::from_millis(10)).map(Message::Tick)
            }*/
            App::IrcConnecting(State {
                post_flag,
                input_value,
                ..
            }) => subscribe_irc::input(*post_flag, input_value, "").map(Message::IrcProgressed),
            _ => Subscription::none(),
        }
    }
    // アプリケーションの表示を操作
    fn view(&mut self) -> Element<Self::Message> {
        match self {
            App::Loading => util::loading_message(),
            App::Loaded(state) | App::IrcFinished(state) | App::IrcConnecting(state) => {
                const MINUTE: u64 = 60;
                const HOUR: u64 = 60 * MINUTE;
                let seconds = state.duration.as_secs();
                let duration = Text::new(format!(
                    "{:0>2}:{:0>2}:{:0>2}",
                    seconds / HOUR,
                    (seconds % HOUR) / MINUTE,
                    seconds % MINUTE
                ))
                .size(8);
                // Scrollable<scrollable::State> => Error
                let scrollable_state: Scrollable<Message> =
                    Scrollable::new(&mut state.scrollable_state)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .push(Text::new(state.display_value.to_string()));
                //static b:button::State = *button;

                let start_irc_button_control: Element<_> = {
                    let (label, toggle, style) = if state.connecting_flag {
                        (
                            "Stop IRC",
                            Message::IrcFinished(Ok(())),
                            style::Button::Stop,
                        )
                    } else {
                        ("Start IRC", Message::IrcStart, style::Button::Start)
                    };
                    Button::new(&mut state.irc_button_state, Text::new(label))
                        .style(style)
                        .on_press(toggle)
                        .into()
                };
                let post_button: Element<_> = {
                    let (label, toggle, style) =
                        ("Post", Message::PostMessage, style::Button::Post);
                    Button::new(&mut state.post_button_state, Text::new(label).size(25))
                        .style(style)
                        .on_press(toggle)
                        .into()
                };
                let input_box = TextInput::new(
                    &mut state.input_state,
                    "Input text...",
                    &mut state.input_value,
                    Message::InputChanged,
                )
                .padding(10)
                .size(15)
                .on_submit(Message::PostMessage);

                let content = Column::new()
                    .padding(10)
                    .spacing(10)
                    .align_items(Align::Start) //.push(duration)
                    .push(start_irc_button_control)
                    .push(Row::new().push(input_box).push(post_button))
                    .push(Row::new().align_items(Align::Center).push(scrollable_state));
                Container::new(content)
                    .width(Length::FillPortion(2))
                    .height(Length::Fill)
                    .into()
            }
        }
    }
}