use std::{borrow::Borrow, fs::File, io::BufWriter, process::exit};

use nix::unistd::{fork, ForkResult};
use tracing::{error, info};

use super::{aof::AofStatus, HdpStatus};
use crate::{db::Db, slot::dict::Dict};

/// 执行snapshot, 替换新的AofStatus
pub fn process(hdp: &mut HdpStatus, slot_id: u16, db: &Db) {
    let lock = match db.slots.get(&slot_id) {
        Some(s) => s,
        None => {
            error!("slot not exists: {}", slot_id);
            return;
        }
    }
    .dict
    .read();
    let base_id = lock.last_write_op_id;
    // fork 子进程做snapshot， 减少持有锁的时间
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child }) => {
            info!(
                "Continuing execution in parent process, new child has pid: {}",
                child
            );
            match AofStatus::new(&hdp.save_hdp_dir, base_id + 1, slot_id) {
                Ok(aof) => {
                    hdp.aof_status_map.insert(slot_id, aof);
                }
                Err(e) => error!(?e),
            }
            drop(lock);
        }
        Ok(ForkResult::Child) => {
            let path = hdp
                .save_hdp_dir
                .join(format!("dump_{}_{}.hds", slot_id, base_id + 1));
            let path_display = &path.display();
            match File::create(&path) {
                Err(why) => error!("couldn't create {}: {}", path_display, why),
                Ok(file) => {
                    let file = BufWriter::new(file);
                    let dict: &Dict = lock.borrow();
                    if let Err(e) = bincode::serialize_into(file, dict) {
                        error!(?e)
                    }
                }
            };
            exit(0);
        }
        Err(e) => error!("Fork failed: {}", e),
    }
}
