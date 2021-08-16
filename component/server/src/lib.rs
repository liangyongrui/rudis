#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::doc_markdown)]
#![allow(unstable_name_collisions)]
#![allow(clippy::semicolon_if_nothing_returned)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::enum_glob_use)]
#![allow(clippy::missing_errors_doc)] //
#![allow(clippy::let_underscore_drop)] //
#![allow(clippy::missing_panics_doc)] //
#![allow(clippy::single_match_else)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::shadow_unrelated)]

/// redis 命令
mod cmd;
// mod utils;

mod connection;
pub use connection::Connection;
pub use parse::frame::Frame;

mod parse;
pub use connection::server;
use parse::{Parse, ParseError};

mod limit;
/// Default port that a redis server listens on.
///
/// Used if no port is specified.
pub const DEFAULT_PORT: &str = "6379";
