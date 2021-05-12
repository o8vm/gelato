#![recursion_limit = "1024"]

extern crate failure;
extern crate futures;
extern crate irc;

pub mod app;
pub mod style;
pub mod util;
pub mod message;
pub mod subscribe_irc;
pub mod content;
pub use content::Content;
pub use app::App;