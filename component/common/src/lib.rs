#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::shadow_unrelated)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::must_use_candidate)]

pub type Error = Box<dyn std::error::Error + Send + Sync>;

/// A specialized `Result` type for rcc operations.
///
/// This is defined as a convenience.
pub type Result<T> = std::result::Result<T, Error>;

pub mod config;
pub mod float;
pub mod options;
pub mod other_type;
pub mod pd_message;
pub mod shutdown;

use std::{
    ops::Bound,
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

/// get now millisecond timestamp
///
/// # Panics
///
/// No panic.
#[inline]
pub fn now_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        // unwrap is safe
        .unwrap()
        .as_millis() as u64
}

pub trait ParseSerdeType<'de, T: Deserialize<'de> + Serialize> {
    fn parse_serde_type(&self) -> T;
}

pub const SYNC_SNAPSHOT: &[u8] = b"*1\r\n$12\r\nsyncsnapshot\r\n";
pub const SYNC_CMD: &[u8] = b"*1\r\n$7\r\nsynccmd\r\n";
pub const SYNC_CMD_PING: &[u8] = b"*1\r\n$11\r\nsynccmdping\r\n";
pub const OK_FRAME: &[u8] = b"+OK\r\n";

pub const SLOT_SIZE: usize = 16384;
