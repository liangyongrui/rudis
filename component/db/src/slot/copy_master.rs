//! 复制master数据的操作

use std::borrow::BorrowMut;

use super::{
    cmd::{self, ExpiresWrite, ExpiresWriteResp, Write},
    Slot,
};
use crate::expire;

impl Slot {
    pub fn call_update<T, C: Write<T> + Clone>(&self, id: u64, cmd: C) {
        let mut dict = self.dict.write();
        if id > dict.last_write_op_id() {
            dict.write_id = id;
            let _ = cmd.apply(dict.borrow_mut());
        }
    }

    pub fn call_expires_update<T, C: ExpiresWrite<T> + Clone>(&self, id: u64, cmd: C) {
        let res = {
            let mut dict = self.dict.write();
            if id <= dict.last_write_op_id() {
                return;
            }
            dict.write_id = id;
            cmd.apply(dict.borrow_mut())
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
    }
}
