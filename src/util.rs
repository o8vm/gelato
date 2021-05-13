use crate::message::Message;
use iced::{Container, Element, Length, Text};
use std::collections::HashMap;

pub fn filter(s: &str, mp: &mut HashMap<String, String>) -> () {
    let v: Vec<&str> = s.split(' ').collect();
    let user: Vec<&str> = v[0].split('!').collect();
    if v[1] == "JOIN" {
        // なぜか更新されない [TODO]
        let x = mp.entry(v[2].to_owned()).or_insert("".to_string());
        *x += user[0];
        *x += " is Joined!\n";
    } else if v[1] == "PRIVMSG" {
        let x = mp.entry(v[2].to_owned()).or_insert("".to_string());
        *x += user[0];
        *x += " ";
        *x += v[3];
    }
}

// メッセージ読み込み中の表示 => Utilにしたい
pub fn loading_message() -> Element<'static, Message> {
    Container::new(Text::new("Loading...").size(50))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_y()
        .into()
}
