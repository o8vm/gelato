#![recursion_limit = "1024"]

extern crate failure;
extern crate futures;
extern crate irc;

pub mod app;
pub mod content;
pub mod message;
pub mod style;
pub mod subscribe_irc;
pub mod util;
pub use app::App;
pub use content::Content;
