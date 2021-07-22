//! slot 的 cmd
//! 写操作，会有个操作id

pub mod kvp;
pub mod simple;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::dict::Dict;

#[derive(Debug, PartialEq, Eq)]
pub struct WriteResp<T> {
    pub payload: T,
    pub new_expires_at: Option<DateTime<Utc>>,
}
pub trait Write<T> {
    fn apply(self, id: u64, dict: &mut Dict) -> crate::Result<WriteResp<T>>;
}

pub trait Read<T> {
    fn apply(self, dict: &Dict) -> crate::Result<T>;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum WriteCmd {
    Set(simple::set::Set),
    None,
}
