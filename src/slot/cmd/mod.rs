//! slot 的 cmd
//! 写操作，会有个操作id

pub mod simple;

use serde::{Deserialize, Serialize};

pub use self::simple::{get, set};
use super::dict::Dict;

pub trait Write<T> {
    fn apply(self, id: u64, dict: &mut Dict) -> crate::Result<T>;
}

pub trait Read<T> {
    fn apply(self, dict: &Dict) -> crate::Result<T>;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum WriteCmd {
    Set(set::Set),
    None,
}
