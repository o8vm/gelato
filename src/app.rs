use iced::{
    button, scrollable, text_input, Align, Application, Button, Clipboard, Column, Command,
    Container, Element, Length, Row, Scrollable, Settings, Subscription, Text, TextInput,
};
use serde::{Deserialize, Serialize};
use std::{
    time::{Duration},
};

// StructでClientを保持するのに必要
/* by Tatsuya Kawanoさん
  1. Clone 実装は必要だけど、 ClientStream の実体は1つにしたい。（複数の所有者で共有させたい）
  2. 非同期ランタイムにもよるが、多くのランタイムはマルチスレッド形式なので、マルチスレッド対応が必要そう
  3. ClientStream はミュータブル（可変）でないとならない。（なぜなら next() を呼ぶ際に &mut self が要求されるから）
  https://github.com/usagi/rust-memory-container-cs
 */
use std::sync::{Arc};
use futures::lock::Mutex;

use crate::model::{message::*, subscribe_irc};
use crate::style;
use crate::util;

use anyhow::Result;

// IRCクライアントをapp.rsで呼び出すなら必要。
use irc::client::prelude::{Client, Config, Sender};
use irc::error::Error;

// iced 2.0ぐらいから、iced::Resultが使えるようになった。
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
    post_flag: bool,
    irc_button_state: button::State,
    post_button_state: button::State,
    scrollable_state: scrollable::State,
    sender: Option<Arc<Mutex<irc::client::Sender>>>,
    client_stream: Option<Arc<Mutex<irc::client::ClientStream>>>,
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
            post_flag: false,
            irc_button_state: button::State::new(),
            post_button_state: button::State::new(),
            scrollable_state: scrollable::State::new(),
            sender: None,
            client_stream: None
        }
    }
}

impl State {
    pub fn new_display_val(s: String) -> Self {
        let mut default: State = State::default();
        default.display_value = String::from(s.to_string());
        default
    }
}

// 下記の実装を元に持ってこられたもの
// https://github.com/hecrj/iced/tree/master/examples/todos
// アプリケーション起動時に設定ファイルの読み込みをする仕組みのためにSavedStateが存在する。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedState {
    pub input_value: String,
    pub display_value: String
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
        /*let config = Config::load("config.toml").unwrap();
        let client = Client::from_config(config).await?;
        client.identify()?;
        let sender = client.sender();
        sender.send_privmsg("#test", "test value dayo").unwrap();
        //client.identify().unwrap();
        //let sender = client.sender();
        //sender.send_privmsg("#test", "test value dayo").unwrap();
        Ok(())*/
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

#[derive(Debug)]
pub enum SaveError {
    // 設定ファイル保存時のエラー状態名
    DirectoryError,
    FileError,
    WriteError,
    FormatError,
    Io(irc::error::Error),
}

// Message構造体が引数にResult<(), Error>を使用するものがある。
// Message構造体はCloneできる必要があるため、引数にReuslt<(), Error>があるなら、これもCloneの必要性がある。
impl Clone for SaveError {
  fn clone(&self) -> Self {
      match *self {
        SaveError::DirectoryError => SaveError::DirectoryError,
        SaveError::FileError => SaveError::FileError,
        SaveError::WriteError => SaveError::WriteError,
        SaveError::FormatError => SaveError::FormatError,
        SaveError::Io(ref err) => SaveError::Io(irc::error::Error::AsyncChannelClosed),
      }
  }
}

#[derive(Debug, Clone)]
pub enum IrcError {
    IrcError,
}

// 試験的実装。IrcClientを取りまとめるstructを作ってみた。
pub struct IrcClient {
  client_stream: irc::client::ClientStream,
  sender: irc::client::Sender,
}

impl IrcClient {
  async fn get_sender() -> Result<IrcClient, failure::Error> {
      let config = Config::load("config.toml").unwrap();
      let mut client = Client::from_config(config).await?;
      client.identify()?;
      Ok(IrcClient{ client_stream: client.stream()?, sender: client.sender()})
  }
}

// アプリケーションの状態。これが大元。ライブラリからも要求される。Stateを内包する設計になっている。
// このenumの分け方とStateの分け方の設計が良いかどうかは若干考えた方がいい。色々と不便の原因にはなっている。
pub enum App {
    Loading,
    Loaded(State),
    IrcConnecting(State),
    IrcFinished(State),
}

// Iced Applicationライブラリが要求する実装
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
        // アプリケーションの種類状態でのマッチ
        match self {
            // アプリケーション初期化中
            App::Loading => {
              match message {
                Message::Loaded(Ok(saved_state)) => {
                  // ここにprintln!を書くと実行されるが、

                  let mut current_state = State::default();//State::new_display_val(saved_state.display_value);
                  async {
                    // ここが実行されない。
                    // https://qiita.com/qnighy/items/05c38f73ef4b9e487ced
                    /*match IrcClient::get_sender().await {
                      Ok(IrcClient{ client_stream, sender}) => {
                        println!("toutatushiteru");
                        current_state.client_stream = Some(Arc::new(Mutex::new(client_stream)));
                        current_state.sender = Some(Arc::new(Mutex::new(sender)));
                      }
                      Err(msg) => println!("failure: {}", msg)
                    };
                    let sender_original = current_state.sender.as_ref().unwrap();
                    {
                      let sender_clone = sender_original.clone();
                      let mut sender = sender_clone.lock().unwrap();
                      sender.send_privmsg("#test", "test value dayo").unwrap();
                    }*/
                    let config = Config::load("config.toml").unwrap();
                    let mut client = Client::from_config(config).await.unwrap();
                    client.identify().unwrap();
                    let sender = client.sender();
                    sender.send_privmsg("#test", "test value dayo").unwrap();
                  };
                  *self = App::Loaded(current_state);
                }
                Message::Loaded(Err(_)) => {
                  *self = App::Loaded(State::default());
                }
                _ => {}
              }
              Command::none()
              //app_loading_command(self, message)
            }
            // アプリケーション初期化完了時
            App::Loaded(state) => {
                let mut saved = false;
                let mut ircflag = false;

                // アプリケーションが受け取ったメッセージごとでフラグを書き換える。
                match message {
                    Message::Saved(_) => {
                        state.saving = false;
                        saved = true;
                    }
                    Message::IrcStart => {
                        ircflag = true;
                    }
                    Message::InputChanged(value) => {
                        println!("Inputを{}に書き換えました", value);
                        state.input_value = value;
                    }
                    _ => {}
                }

                // フラグに基づき、最終的なコマンドを設定する。
                // TODOサンプルを元に作成しているため、saved, dirtyはtodoからきている。使われていないものもある。
                if !saved {
                    state.dirty = true;
                }

                if state.dirty && !state.saving {
                    state.dirty = false;
                    state.saving = true;
                    // いったん何もしないことにする。
                    /*Command::perform(
                        SavedState {
                            input_value: state.input_value.clone(),
                            display_value: state.display_value.clone(),
                        }
                        .save(),
                        Message::Saved,
                    )*/
                    Command::none()
                  // IRCが開始されたら、selfをIRCConnectingに強制上書きする。
                  } else if ircflag {
                    let mut state_edit = state.clone();
                    *self = App::IrcConnecting(state_edit);
                    Command::none()
                } else {
                    Command::none()
                }
            }
            // IRC接続状態の時
            App::IrcConnecting(state) => {
                state.connecting_flag = true;
                let mut irc_finished = false;
                let mut posted = false;
                match message {
                    // Message::IrcProgressedは、subscription関数のmapで渡されている関数
                    Message::IrcProgressed(progress_state) => match progress_state {
                        // model/subscribe_irc.rsで実装されているProgressから結果のmessage_textが返却される。
                        subscribe_irc::Progress::Advanced(message_text) => {
                            // メッセージのフィルタリング
                            let filtered_text: &str = util::filter(&message_text);
                            state.display_value.push_str(filtered_text);
                        }
                        subscribe_irc::Progress::Finished => {
                            irc_finished = true;
                        }
                        subscribe_irc::Progress::Errored => {
                            irc_finished = true;
                        }
                        _ => {}
                    },
                    Message::IrcFinished(_) => {
                        irc_finished = true;
                        state.connecting_flag = false;
                    }
                    Message::InputChanged(value) => {
                        println!("Inputを{}に書き換えました", value);
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
                      Ok(IrcClient{ client_stream, sender}) => {
                        println!("toutatushiteru");
                        sender.send_privmsg("#test", "test value dayo").unwrap();
                      }
                      Err(msg) => println!("failure: {}", msg)
                    }
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

    // サブスクリプションの登録。
    // selfはアプリケーションのenumのため、必要に応じてStateの中身を取り出す。
    fn subscription(&self) -> Subscription<Message> {
        match self {
            // IRCと接続している時の非同期通信を設定。
            // 具体的な実装は、model/subscribe_irc.rsで担当する。
            // なんとかして、GUIからの入力値を渡そうとしている。
            // GUIからの入力値をIRCサーバーに送信する方法は2種類考えられる。
            // 1. app.rs内部で送信してしまう。
            // 2. model/subscribe_irc.rsになんとかしてGUIからの入力値ならびに送信フラグを渡す。

            // 1.を実現したいのであれば、何らかの方法でclientオブジェクトをapp.rsが保持しないといけないが、できていない。
            // 2.を実現したのであれば、何らかの方法でsubscribe_ircにGUIからの入力値を渡さないといけない。
            // subscribe_irc.rs内部で作成したクライアントからsendすること自体は可能であることを確認した。
            App::IrcConnecting(State {
                post_flag,
                input_value,
                ..
            }) => subscribe_irc::input(*post_flag, input_value, "").map(Message::IrcProgressed),
            // input関数への受け渡しは適当にいろいろ試しているため、何も考えていない。

            // IRCと接続時以外は特に何もしない。
            _ => Subscription::none(),
        }
    }

    // 更新された時に呼び出される描画関数
    // いつか部分ごとに関数化&外部ファイル化したいと考えているが、現状はここに全部書いている。
    // CSSのような装飾はstyle.rsで設定している。
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