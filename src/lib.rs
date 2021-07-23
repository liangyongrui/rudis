pub mod cmd;
mod cmd_reader;
mod config;
/// 过期处理
mod expire;
/// 请求转发
mod forward;
mod replica;
mod slot;
mod utils;
pub use cmd::Command;

mod connection;
pub use connection::Connection;
pub use parse::frame::Frame;

// mod db;
/// 暂时pub
pub mod db2;
// pub use db::data_type::SimpleType;
// use db::Db;
use db2::Db;
mod parse;
pub use connection::server;
pub use parse::to_bytes::ToVecU8;
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
