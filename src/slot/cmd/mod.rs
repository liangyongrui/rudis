//! slot 的 cmd
//! 写操作，会有个操作id

pub mod deque;
pub mod kvp;
pub mod set;
pub mod simple;
pub mod sorted_set;

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
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
    fn apply(self, dict: &RwLock<Dict>) -> crate::Result<T>;
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
    SortedSetAdd(sorted_set::add::Req),
    SortedSetRemove(sorted_set::remove::Req),
    SortedSetRemoveByRankRange(sorted_set::remove_by_rank_range::Req),
    SortedSetRemoveByScoreRange(sorted_set::remove_by_score_range::Req),
    SortedSetRemoveByLexRange(sorted_set::remove_by_lex_range::Req),
    None,
}
