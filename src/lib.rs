#![recursion_limit = "1024"]

extern crate failure;
extern crate futures;
extern crate irc;

pub mod app;
pub mod style;
pub mod util;
pub mod message;
pub mod subscribe_irc;
pub use app::App;