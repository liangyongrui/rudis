//! slot 层
//!
//! 实现内部的命令 不需要兼容redis
//! slot 层的操作，加锁的时间粒度降为最小

use std::borrow::BorrowMut;

use parking_lot::RwLock;
use rpds::{HashTrieMapSync, HashTrieSetSync};

use self::{
    cmd::WriteResp,
    data_type::SimpleType,
    dict::{Dict, Value},
};
use crate::{
    db::BgTask,
    forward,
    slot::cmd::{Read, Write},
};

pub mod cmd;
pub mod data_type;
pub mod dict;

pub struct Slot {
    slot_id: u16,
    pub dict: RwLock<Dict>,
    bg_task: BgTask,
}

impl Slot {
    pub fn new(slot_id: u16, bg_task: BgTask) -> Self {
        // todo load from disk
        Self {
            slot_id,
            dict: RwLock::new(Dict::new()),
            bg_task,
        }
    }

    /// 更新整个 dict
    pub fn update_dict(&self, dict: Dict) {
        // todo
    }

    fn call_write<T, C: Write<T> + Clone>(&self, cmd: C) -> crate::Result<T> {
        // 加锁执行命令
        let (res, id) = {
            let mut dict = self.dict.write();
            let dict = dict.borrow_mut();
            let id = dict.next_id();
            (cmd.clone().apply(id, dict), id)
        };

        let res = match res {
            Ok(WriteResp {
                new_expires_at,
                payload,
            }) => {
                // 设置自动过期
                if let Some((ea, key)) = new_expires_at {
                    let _ = self.bg_task.expire_sender.send(crate::expire::Entry {
                        expires_at: ea,
                        slot: self.slot_id,
                        id,
                        key,
                    });
                }
                Ok(payload)
            }
            Err(e) => Err(e),
        };

        // 转发执行完成的请求
        let _ = self.bg_task.forward_sender.send(forward::Message {
            id,
            slot: self.slot_id,
            cmd: cmd.into(),
        });

        res
    }
}

/// 写命令
impl Slot {
    pub fn set(&self, cmd: cmd::simple::set::Req) -> crate::Result<SimpleType> {
        self.call_write(cmd)
    }
    pub fn del(&self, cmd: cmd::simple::del::Req) -> crate::Result<Option<Value>> {
        self.call_write(cmd)
    }
    pub fn expire(&self, cmd: cmd::simple::expire::Req) -> crate::Result<bool> {
        self.call_write(cmd)
    }
    pub fn incr(&self, cmd: cmd::simple::incr::Req) -> crate::Result<i64> {
        self.call_write(cmd)
    }
    pub fn kvp_set(&self, cmd: cmd::kvp::set::Req) -> crate::Result<cmd::kvp::set::Resp> {
        self.call_write(cmd)
    }
    pub fn kvp_del(&self, cmd: cmd::kvp::del::Req) -> crate::Result<cmd::kvp::del::Resp> {
        self.call_write(cmd)
    }
    pub fn kvp_incr(&self, cmd: cmd::kvp::incr::Req) -> crate::Result<i64> {
        self.call_write(cmd)
    }
    pub fn deque_push(&self, cmd: cmd::deque::push::Req) -> crate::Result<cmd::deque::push::Resp> {
        self.call_write(cmd)
    }
    pub fn deque_pop(&self, cmd: cmd::deque::pop::Req) -> crate::Result<Vec<SimpleType>> {
        self.call_write(cmd)
    }
    pub fn set_add(&self, cmd: cmd::set::add::Req) -> crate::Result<cmd::set::add::Resp> {
        self.call_write(cmd)
    }
    pub fn set_remove(&self, cmd: cmd::set::remove::Req) -> crate::Result<cmd::set::remove::Resp> {
        self.call_write(cmd)
    }
    pub fn sorted_set_add(
        &self,
        cmd: cmd::sorted_set::add::Req,
    ) -> crate::Result<cmd::sorted_set::add::Resp> {
        self.call_write(cmd)
    }
    pub fn sorted_set_remove(
        &self,
        cmd: cmd::sorted_set::remove::Req,
    ) -> crate::Result<cmd::sorted_set::remove::Resp> {
        self.call_write(cmd)
    }
    pub fn sorted_set_remove_by_lex_range(
        &self,
        cmd: cmd::sorted_set::remove_by_lex_range::Req,
    ) -> crate::Result<Vec<data_type::sorted_set::Node>> {
        self.call_write(cmd)
    }
    pub fn sorted_set_remove_by_rank_range(
        &self,
        cmd: cmd::sorted_set::remove_by_rank_range::Req,
    ) -> crate::Result<Vec<data_type::sorted_set::Node>> {
        self.call_write(cmd)
    }
    pub fn sorted_set_remove_by_score_range(
        &self,
        cmd: cmd::sorted_set::remove_by_score_range::Req,
    ) -> crate::Result<Vec<data_type::sorted_set::Node>> {
        self.call_write(cmd)
    }
}

/// 读命令
impl Slot {
    pub fn get(&self, cmd: cmd::simple::get::Req<'_>) -> crate::Result<SimpleType> {
        cmd.apply(&self.dict)
    }
    pub fn exists(&self, cmd: cmd::simple::exists::Req<'_>) -> crate::Result<bool> {
        cmd.apply(&self.dict)
    }
    pub fn kvp_exists(&self, cmd: cmd::kvp::exists::Req<'_>) -> crate::Result<bool> {
        cmd.apply(&self.dict)
    }
    pub fn kvp_get(&self, cmd: cmd::kvp::get::Req<'_>) -> crate::Result<SimpleType> {
        cmd.apply(&self.dict)
    }
    pub fn kvp_get_all(
        &self,
        cmd: cmd::kvp::get_all::Req<'_>,
    ) -> crate::Result<Option<HashTrieMapSync<SimpleType, SimpleType>>> {
        cmd.apply(&self.dict)
    }
    pub fn deque_len(&self, cmd: cmd::deque::len::Req<'_>) -> crate::Result<usize> {
        cmd.apply(&self.dict)
    }
    pub fn deque_range(&self, cmd: cmd::deque::range::Req<'_>) -> crate::Result<Vec<SimpleType>> {
        cmd.apply(&self.dict)
    }
    pub fn set_exists(&self, cmd: cmd::set::exists::Req<'_>) -> crate::Result<bool> {
        cmd.apply(&self.dict)
    }
    pub fn set_get_all(
        &self,
        cmd: cmd::set::get_all::Req<'_>,
    ) -> crate::Result<Option<HashTrieSetSync<SimpleType>>> {
        cmd.apply(&self.dict)
    }
    pub fn sorted_set_range_by_lex(
        &self,
        cmd: cmd::sorted_set::range_by_lex::Req<'_>,
    ) -> crate::Result<Vec<data_type::sorted_set::Node>> {
        cmd.apply(&self.dict)
    }
    pub fn sorted_set_range_by_rank(
        &self,
        cmd: cmd::sorted_set::range_by_rank::Req<'_>,
    ) -> crate::Result<Vec<data_type::sorted_set::Node>> {
        cmd.apply(&self.dict)
    }
    pub fn sorted_set_range_by_score(
        &self,
        cmd: cmd::sorted_set::range_by_score::Req<'_>,
    ) -> crate::Result<Vec<data_type::sorted_set::Node>> {
        cmd.apply(&self.dict)
    }
    pub fn sorted_set_rank(
        &self,
        cmd: cmd::sorted_set::rank::Req<'_>,
    ) -> crate::Result<Option<usize>> {
        cmd.apply(&self.dict)
    }
}
