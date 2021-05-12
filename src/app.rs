use crate::{content, message::*, style, subscribe_irc, util};
use iced::{
    button, pane_grid, text_input, Align, Application, Button, Clipboard, Column, Command,
    Container, Element, Length, PaneGrid, Row, Settings, Subscription, Text, TextInput,
};
use serde::{Deserialize, Serialize};

use irc::client::prelude::{Client, Config};
use std::sync::Arc;

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
    current_channel: String,
    config: Config,
    irc_button_state: button::State,
    post_button_state: button::State,
    sender: Option<Arc<futures::lock::Mutex<irc::client::Sender>>>,
    panes: pane_grid::State<content::Content>,
    panes_created: usize,
    focus: Option<pane_grid::Pane>,
    client_stream: Option<Arc<futures::lock::Mutex<irc::client::ClientStream>>>,
}

impl Default for State {
    fn default() -> Self {
        let (panes, _) = pane_grid::State::new(content::Content::new(0));
        Self {
            input_state: text_input::State::new(),
            input_value: String::from(""),
            connecting_flag: false,
            display_value: String::from(""),
            saving: true,
            dirty: true,
            current_channel: String::from("#channel-name"),
            config: Config::default(),
            irc_button_state: button::State::new(),
            post_button_state: button::State::new(),
            sender: None,
            panes: panes,
            panes_created: 1,
            focus: None,
            client_stream: None,
        }
    }
}

impl State {
    pub fn new_display_val(s: String) -> Self {
        State {
            display_value: s,
            ..Default::default()
        }
    }
}

// 下記の実装を元に持ってこられたもの
// https://github.com/hecrj/iced/tree/master/examples/todos
// アプリケーション起動時に設定ファイルの読み込みをする仕組みのためにSavedStateが存在する。
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

// 試験的実装。IrcClientを取りまとめるstructを作ってみた。
pub struct IrcClient {
    client_stream: irc::client::ClientStream,
    sender: irc::client::Sender,
}

impl IrcClient {
    async fn get_client(state: &mut State) -> Result<IrcClient, failure::Error> {
        let config = Config::load("config.toml").unwrap();
        state.config = config.clone();
        let mut client = Client::from_config(config).await?;
        client.identify()?;
        Ok(IrcClient {
            client_stream: client.stream()?,
            sender: client.sender(),
        })
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
                        let mut current_state = State::default();

                        // futures::executor::block_on関数を使わないと、asyncの括弧の中が実行されない。
                        futures::executor::block_on(async {
                            let irc_client_struct = IrcClient::get_client(&mut current_state)
                                .await
                                .expect("get_client()");
                            current_state.client_stream = Some(Arc::new(
                                futures::lock::Mutex::new(irc_client_struct.client_stream),
                            ));
                            current_state.sender = Some(Arc::new(futures::lock::Mutex::new(
                                irc_client_struct.sender,
                            )));
                        });
                        *self = App::Loaded(current_state);
                    }
                    Message::Loaded(Err(_)) => {
                        *self = App::Loaded(State::default());
                    }
                    _ => {}
                }
                Command::none()
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
                        state.input_value = value;
                    }
                    Message::Split(axis, pane) => {
                        let result = state.panes.split(
                            axis,
                            &pane,
                            content::Content::new(state.panes_created),
                        );

                        if let Some((pane, _)) = result {
                            state.focus = Some(pane);
                        }

                        state.panes_created += 1;
                    }
                    Message::SplitFocused(axis) => {
                        if let Some(pane) = state.focus {
                            let result = state.panes.split(
                                axis,
                                &pane,
                                content::Content::new(state.panes_created),
                            );

                            if let Some((pane, _)) = result {
                                state.focus = Some(pane);
                            }

                            state.panes_created += 1;
                        }
                    }
                    Message::FocusAdjacent(direction) => {
                        if let Some(pane) = state.focus {
                            if let Some(adjacent) = state.panes.adjacent(&pane, direction) {
                                state.focus = Some(adjacent);
                            }
                        }
                    }
                    Message::Clicked(pane) => {
                        state.focus = Some(pane);
                    }
                    Message::Resized(pane_grid::ResizeEvent { split, ratio }) => {
                        state.panes.resize(&split, ratio);
                    }
                    Message::Dragged(pane_grid::DragEvent::Dropped { pane, target }) => {
                        state.panes.swap(&pane, &target);
                    }
                    Message::Dragged(_) => {}
                    Message::Close(pane) => {
                        if let Some((_, sibling)) = state.panes.close(&pane) {
                            state.focus = Some(sibling);
                        }
                    }
                    Message::CloseFocused => {
                        if let Some(pane) = state.focus {
                            if let Some((_, sibling)) = state.panes.close(&pane) {
                                state.focus = Some(sibling);
                            }
                        }
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
                    *self = App::IrcConnecting(state.clone());
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
                let mut input_word = String::from("");
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
                        state.input_value = value;
                    }
                    Message::PostMessage => {
                        posted = true;
                        input_word.push_str(&state.input_value.clone());
                        state.input_value = String::from("");
                    }
                    _ => {}
                }

                if posted && !input_word.is_empty() {
                    let sender_original = Arc::clone(&state.sender.as_ref().unwrap());
                    let call = async move {
                        let sender = sender_original.lock().await;
                        (*sender).send_privmsg("#test", input_word).unwrap();
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
            App::IrcConnecting(State { client_stream, .. }) => {
                let client_stream = client_stream.as_ref();
                subscribe_irc::input(client_stream, "").map(Message::IrcProgressed)
            }
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
                // I'm going to delete START IRC BUTTON
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
                    Button::new(&mut state.irc_button_state, Text::new(label).size(25))
                        .style(style)
                        .on_press(toggle)
                        .into()
                };

                // Below Contents
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
                    &state.input_value,
                    Message::InputChanged,
                )
                .padding(10)
                .size(15)
                .on_submit(Message::PostMessage);

                let content2 = Column::new()
                    .padding(10)
                    .spacing(10)
                    .align_items(Align::Start)
                    .push(
                        Row::new()
                            .push(input_box)
                            .push(post_button)
                            .push(start_irc_button_control),
                    );

                // Panel Grid Components
                let focus = state.focus;
                let total_panes = state.panes.len();
                let channel_name = state.current_channel.to_string();
                let text = state.display_value.clone();

                let pane_grid = PaneGrid::new(&mut state.panes, |pane, content| {
                    let is_focused = focus == Some(pane);

                    let title = Row::with_children(vec![Text::new(channel_name.to_owned()).into()])
                        .spacing(5);

                    let title_bar = pane_grid::TitleBar::new(title)
                        .padding(5)
                        .style(style::TitleBar { is_focused });
                    pane_grid::Content::new(content.view(pane, total_panes, text.to_owned()))
                        .title_bar(title_bar)
                        .style(style::Pane { is_focused })
                })
                .width(Length::Fill)
                .height(Length::Fill)
                .spacing(10)
                .on_click(Message::Clicked)
                .on_drag(Message::Dragged)
                .on_resize(10, Message::Resized);

                // Container Components
                let container = Column::new().spacing(1).push(pane_grid).push(content2);

                Container::new(container)
                    .width(Length::FillPortion(2))
                    .height(Length::Fill)
                    .into()
            }
        }
    }
}
