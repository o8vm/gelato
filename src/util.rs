use crate::model::message::Message;
use iced::{
  Container, Element, Length, Text,
};

pub fn filter(s: &str) -> &str {
  let v: Vec<&str> = s.split(' ').collect();
  if v[1] == "PONG" || v[0] == "PING" {
    ""
  } else {
    s
  }
}

// メッセージ読み込み中の表示 => Utilにしたい
pub fn loading_message() -> Element<'static, Message> {
Container::new(
    Text::new("Loading...")
        .size(50),
)
.width(Length::Fill)
.height(Length::Fill)
.center_y()
.into()
}