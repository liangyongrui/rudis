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

mod shutdown;
use shutdown::Shutdown;

/// Default port that a redis server listens on.
///
/// Used if no port is specified.
pub const DEFAULT_PORT: &str = "6379";

pub type Error = Box<dyn std::error::Error + Send + Sync>;

/// A specialized `Result` type for rcc operations.
///
/// This is defined as a convenience.
pub type Result<T> = std::result::Result<T, Error>;
