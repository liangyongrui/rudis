pub type Error = Box<dyn std::error::Error + Send + Sync>;

/// A specialized `Result` type for rcc operations.
///
/// This is defined as a convenience.
pub type Result<T> = std::result::Result<T, Error>;

pub mod config;
pub mod float;
pub mod options;
pub mod other_type;
pub mod shutdown;

use std::{
    ops::Bound,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
pub trait BoundExt<T> {
    fn map<O, F: FnOnce(T) -> O>(self, f: F) -> Bound<O>;
}

impl<T> BoundExt<T> for Bound<T> {
    #[inline]
    fn map<O, F: FnOnce(T) -> O>(self, f: F) -> Bound<O> {
        match self {
            Bound::Included(t) => Bound::Included(f(t)),
            Bound::Excluded(t) => Bound::Excluded(f(t)),
            Bound::Unbounded => Bound::Unbounded,
        }
    }
}

#[inline]
pub fn now_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[inline]
pub fn u8_to_string(data: &[u8]) -> Arc<str> {
    std::str::from_utf8(data)
        .map(|s| s.into())
        .expect("protocol error; invalid string")
}

#[inline]
pub fn u8_to_i64(data: &[u8]) -> i64 {
    atoi::atoi::<i64>(data).expect("protocol error; invalid number")
}

pub trait ParseSerdeType<'de, T: Deserialize<'de> + Serialize> {
    fn parse_serde_type(&self) -> T;
}

pub const SYNC_SNAPSHOT: &[u8] = b"*1\r\n$12\r\nsyncsnapshot\r\n";
pub const SYNC_CMD: &[u8] = b"*1\r\n$7\r\nsynccmd\r\n";
pub const SYNC_CMD_PING: &[u8] = b"*1\r\n$11\r\nsynccmdping\r\n";
pub const OK_FRAME: &[u8] = b"+OK\r\n";
