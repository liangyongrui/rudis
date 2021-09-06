//! slot 的 cmd
//! 写操作，会有个操作id

pub mod deque;
pub mod kvp;
pub mod set;
pub mod simple;
pub mod sorted_set;

use keys::Key;
use serde::{Deserialize, Serialize};

use crate::Dict;

#[derive(Debug, PartialEq, Eq)]
pub struct ExpiresWriteResp<T> {
    pub payload: T,
    pub expires_status: ExpiresStatus,
}

/// 当不需要更新过期时间时
/// 这个为None
#[derive(Debug, PartialEq, Eq)]
pub enum ExpiresStatus {
    None,
    Update(ExpiresStatusUpdate),
}

/// 删除before 添加new
#[derive(Debug, PartialEq, Eq)]
pub struct ExpiresStatusUpdate {
    pub key: Key,
    pub before: u64,
    pub new: u64,
}
pub trait ExpiresWrite<T>
where
    Self: Into<WriteCmd>,
{
    /// apply with expire time
    /// # Errors
    /// inner error
    fn apply(self, dict: &mut Dict) -> common::Result<ExpiresWriteResp<T>>;
}
pub trait Write<T>
where
    Self: Into<WriteCmd>,
{
    /// apply
    /// # Errors
    /// inner error
    fn apply(self, dict: &mut Dict) -> common::Result<T>;
}

pub trait Read<T> {
    /// apply
    /// # Errors
    /// inner error
    fn apply(self, dict: &Dict) -> common::Result<T>;
}

#[allow(clippy::module_name_repetitions)]
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
    SetRemove(set::remove::Req),
    SortedSetAdd(sorted_set::add::Req),
    SortedSetRemove(sorted_set::remove::Req),
    SortedSetRemoveByRankRange(sorted_set::remove_by_rank_range::Req),
    SortedSetRemoveByScoreRange(sorted_set::remove_by_score_range::Req),
    SortedSetRemoveByLexRange(sorted_set::remove_by_lex_range::Req),
    // 心跳返回值也用这个
    None,
}
