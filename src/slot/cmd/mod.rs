//! slot 的 cmd
//! 写操作，会有个操作id

pub mod deque;
pub mod kvp;
pub mod set;
pub mod simple;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{data_type::SimpleType, dict::Dict};

#[derive(Debug, PartialEq, Eq)]
pub struct WriteResp<T> {
    pub payload: T,
    /// # WARN
    /// 当不需要更新过期时间时
    /// - 这个为None
    /// - 且 value 的id 不能更新, 避免自动过期失效
    pub new_expires_at: Option<(DateTime<Utc>, SimpleType)>,
}
pub trait Write<T>
where
    Self: Into<WriteCmd>,
{
    fn apply(self, id: u64, dict: &mut Dict) -> crate::Result<WriteResp<T>>;
}

pub trait Read<T> {
    fn apply(self, dict: &Dict) -> crate::Result<T>;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum WriteCmd {
    Del(simple::del::Req),
    Expire(simple::expire::Req),
    Incr(simple::incr::Req),
    Set(simple::set::Req),
    KvpDel(kvp::del::Req),
    KvpIncr(kvp::incr::Req),
    KvpSet(kvp::set::Req),
    DequePop(deque::pop::Req),
    DequePush(deque::push::Req),
    SetAdd(set::add::Req),
    None,
}
