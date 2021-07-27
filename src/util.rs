use crate::message::Message;
use iced::{Container, Element, Length, Text};
use std::collections::HashMap;

pub fn filter(s: &str, mp: &mut HashMap<String, String>) -> () {
    let space_split_vec: Vec<&str> = s.splitn(4, ' ').collect();
    if space_split_vec.len() >= 4 {
        let head: Vec<&str> = space_split_vec[0].splitn(2, '!').collect();
        let user = head[0].strip_prefix(':').unwrap_or("");
        let op = space_split_vec[1];
        let channel = space_split_vec[2];
        let text = space_split_vec[3];

        if op == "JOIN" {
            // なぜか更新されない [TODO]
            let x = mp.entry(channel.to_owned()).or_insert("".to_string());
            *x += user;
            *x += " is Joined!\n";
        } else if op == "PRIVMSG" {
            let x = mp.entry(channel.to_owned()).or_insert("".to_string());
            *x += user;
            *x += "  ";
            *x += text;
        }
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
