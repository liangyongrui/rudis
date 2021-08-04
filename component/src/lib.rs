#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::doc_markdown)]
#![allow(unstable_name_collisions)]
#![allow(clippy::semicolon_if_nothing_returned)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::module_name_repetitions)] // 以后去掉
#![allow(clippy::enum_glob_use)]
#![allow(clippy::missing_errors_doc)] //
#![allow(clippy::let_underscore_drop)] //
#![allow(clippy::missing_panics_doc)] //
#![allow(clippy::single_match_else)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::shadow_unrelated)]

/// 子进程管理
mod child_process;
/// redis 命令
mod cmd;
mod config;
/// 过期处理
mod expire;
/// 请求转发
mod forward;
/// Hard disk persistence
mod hdp;
mod replica;
mod slot;
mod utils;

mod connection;
pub use connection::Connection;
pub use parse::frame::Frame;

mod db;
use crate::db::Db;

mod parse;
pub use connection::server;
use parse::{Parse, ParseError};

mod limit;
mod shutdown;
/// Default port that a redis server listens on.
///
/// Used if no port is specified.
pub const DEFAULT_PORT: &str = "6379";

pub type Error = Box<dyn std::error::Error + Send + Sync>;

/// A specialized `Result` type for rcc operations.
///
/// This is defined as a convenience.
pub type Result<T> = std::result::Result<T, Error>;
