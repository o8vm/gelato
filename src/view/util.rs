use iced::{
  Align, Column, Container, Element, Length, Text
};
use super::Message;

// 引数の文字列をアプリケーションの画面に表示
pub fn show_display_val(display_value: &str) -> Element<'static, Message> {
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