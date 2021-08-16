use std::{borrow::Borrow, fs::File, io::BufWriter, process::exit, sync::atomic::AtomicBool};

use dict::Dict;
use nix::unistd::{fork, ForkResult};
use tracing::{error, info};

use crate::{
    child_process,
    hdp::{self, aof},
    Db,
};

pub static IN_PROGRESS: AtomicBool = AtomicBool::new(false);

/// 执行snapshot, 替换新的AofStatus
pub fn process(hdp: &mut hdp::Status, slot_id: u16, db: &Db) {
    let lock = match db.slots.get(&slot_id) {
        Some(s) => s,
        None => {
            error!("slot not exists: {}", slot_id);
            return;
        }
    }
    .dict
    .read();
    let base_id = lock.last_write_op_id();
    // fork 子进程做snapshot， 减少持有锁的时间
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child }) => {
            info!(
                "Continuing execution in parent process, new child has pid: {}",
                child
            );
            match aof::Status::new(&hdp.save_hdp_dir, base_id + 1, slot_id) {
                Ok(aof) => {
                    hdp.aof_status_map.insert(slot_id, aof);
                }
                Err(e) => error!(?e),
            }
            drop(lock);
            child_process::add(child, child_process::Info::HdpSnapshot { base_id });
        }
        Ok(ForkResult::Child) => {
            let path = hdp
                .save_hdp_dir
                .join(format!("{}/dump_{}.ss", base_id + 1, slot_id));
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
