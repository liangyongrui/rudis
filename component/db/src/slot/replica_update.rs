//! 复制master数据的操作

use std::{borrow::BorrowMut, cmp::Ordering};

use dict::cmd::WriteCmd;

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
    fn call_update<T, C: Write<T> + Clone>(&self, id: u64, cmd: C) -> Ordering {
        if let Some(s) = &mut *self.share_status.write() {
            match id.cmp(&(s.dict.last_write_op_id() + 1)) {
                Ordering::Less => Ordering::Less,
                Ordering::Equal => {
                    s.dict.write_id = id;
                    let _ = cmd.apply(s.dict.borrow_mut());
                    Ordering::Equal
                }
                Ordering::Greater => Ordering::Greater,
            }
        } else {
            Ordering::Greater
        }
    }

    pub fn call_expires_update<T, C: ExpiresWrite<T> + Clone>(&self, id: u64, cmd: C) -> Ordering {
        let res = if let Some(s) = &mut *self.share_status.write() {
            match id.cmp(&(s.dict.last_write_op_id() + 1)) {
                Ordering::Less => return Ordering::Less,
                Ordering::Equal => {
                    s.dict.write_id = id;
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
                        let _ = self.bg_task.expire_sender.send(expire::Message::Update(
                            expire::Update {
                                status: u,
                                slot: self.slot_id,
                            },
                        ));
                    }
                }
            }
        }
        Ordering::Equal
    }
}
