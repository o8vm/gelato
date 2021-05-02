extern crate futures;
extern crate gelato;
extern crate irc;

use iced::{Application, Settings};

fn main() -> iced::Result {
    let mut sets = Settings::default();
    sets.default_font = Some(include_bytes!(
        "../fonts/M_PLUS_Rounded_1c/MPLUSRounded1c-Medium_reverted.ttf"
    ));
    gelato::App::run(sets)
}
