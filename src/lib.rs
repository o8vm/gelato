#![recursion_limit = "1024"]

extern crate failure;
extern crate futures;
extern crate irc;

pub mod app;
pub mod model;
pub mod style;
pub mod util;
pub use app::App;
