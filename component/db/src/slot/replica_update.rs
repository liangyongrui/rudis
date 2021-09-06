//! 复制master数据的操作

use std::{borrow::BorrowMut, cmp::Ordering};

use dict::{cmd::WriteCmd, Dict, MemDict};
use tracing::error;

use super::{
    cmd::{self, ExpiresWrite, ExpiresWriteResp, Write},
    Slot,
};
use crate::expire;

impl Slot {
    pub fn process_forward(&self, id: u64, cmd: WriteCmd) -> Ordering {
        match cmd {
            cmd::WriteCmd::Del(req) => self.call_expires_update(id, req),
            cmd::WriteCmd::Expire(req) => self.call_expires_update(id, req),
            cmd::WriteCmd::Incr(req) => self.call_update(id, req),
            cmd::WriteCmd::Set(req) => self.call_expires_update(id, req),
            cmd::WriteCmd::KvpDel(req) => self.call_update(id, req),
            cmd::WriteCmd::KvpIncr(req) => self.call_update(id, req),
            cmd::WriteCmd::KvpSet(req) => self.call_update(id, req),
            cmd::WriteCmd::DequePop(req) => self.call_update(id, req),
            cmd::WriteCmd::DequePush(req) => self.call_update(id, req),
            cmd::WriteCmd::SetAdd(req) => self.call_update(id, req),
            cmd::WriteCmd::SetRemove(req) => self.call_update(id, req),
            cmd::WriteCmd::SortedSetAdd(req) => self.call_update(id, req),
            cmd::WriteCmd::SortedSetRemove(req) => self.call_update(id, req),
            cmd::WriteCmd::SortedSetRemoveByRankRange(req) => self.call_update(id, req),
            cmd::WriteCmd::SortedSetRemoveByScoreRange(req) => self.call_update(id, req),
            cmd::WriteCmd::SortedSetRemoveByLexRange(req) => self.call_update(id, req),
            cmd::WriteCmd::None => Ordering::Equal,
        }
    }

    fn call_update<T, C: Write<T, MemDict> + Clone>(&self, id: u64, cmd: C) -> Ordering {
        self.share_status
            .write()
            .as_mut()
            .map_or(Ordering::Greater, |s| {
                match id.cmp(&(s.dict.last_write_op_id() + 1)) {
                    Ordering::Less => Ordering::Less,
                    Ordering::Equal => {
                        s.dict.set_write_id(id);
                        if let Err(e) = cmd.apply(s.dict.borrow_mut()) {
                            error!("call update: {:?}", e);
                        }
                        Ordering::Equal
                    }
                    Ordering::Greater => Ordering::Greater,
                }
            })
    }

    pub fn call_expires_update<T, C: ExpiresWrite<T, MemDict> + Clone>(
        &self,
        id: u64,
        cmd: C,
    ) -> Ordering {
        let res = if let Some(s) = &mut *self.share_status.write() {
            match id.cmp(&(s.dict.last_write_op_id() + 1)) {
                Ordering::Less => return Ordering::Less,
                Ordering::Equal => {
                    s.dict.set_write_id(id);
                    cmd.apply(&mut s.dict)
                }
                Ordering::Greater => return Ordering::Greater,
            }
        } else {
            return Ordering::Greater;
        };

        if let Ok(ExpiresWriteResp { expires_status, .. }) = res {
            match expires_status {
                cmd::ExpiresStatus::None => (),
                cmd::ExpiresStatus::Update(u) => {
                    if u.before != u.new {
                        if let Err(e) = self.bg_task.expire_sender.send(expire::Message::Update(
                            expire::Update {
                                status: u,
                                slot: self.slot_id,
                            },
                        )) {
                            error!("call_expires_update: {:?}", e);
                        }
                    }
                }
            }
        }
        Ordering::Equal
    }
}
