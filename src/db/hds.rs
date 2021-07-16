use std::{
    collections::HashMap,
    fs::File,
    io::BufWriter,
    path::Path,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use nix::unistd::{fork, ForkResult};
use tokio::time::sleep;
use tracing::{error, info};

use super::{slot::Slot, Db};
use crate::config::CONFIG;

#[derive(Debug)]
pub struct HdsStatus {
    can_do: AtomicBool,
    update_time: Instant,
    change_times: AtomicU64,
}

impl HdsStatus {
    pub fn new() -> Self {
        Self {
            can_do: AtomicBool::new(false),
            update_time: Instant::now(),
            change_times: AtomicU64::new(0),
        }
    }
}

pub async fn run_bg_save_task(db: Arc<Db>) {
    // todo 安全退出
    loop {
        sleep(Duration::from_secs(1)).await;
        let load = db.hds_status.load();
        if !load.can_do.load(Ordering::SeqCst) {
            continue;
        }
        let duration_now = Instant::now() - load.update_time;
        let mut trigger = false;
        for (duration, times) in &CONFIG.save {
            if &duration_now > duration && load.change_times.load(Ordering::SeqCst) > *times {
                trigger = true;
                break;
            }
        }
        if trigger {
            drop(load);
            db.hds_status.swap(Arc::new(HdsStatus::new()));
            save_slots(&db.slots);
            let _res = db.hds_status.load().can_do.compare_exchange(
                false,
                true,
                Ordering::SeqCst,
                Ordering::SeqCst,
            );
        }
    }
}
pub fn save_slots(slots: &HashMap<u16, Slot>) {
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child, .. }) => {
            info!(
                "Continuing execution in parent process, new child has pid: {}",
                child
            );
        }
        Ok(ForkResult::Child) => {
            for (id, slot) in slots {
                save_slot(*id, slot);
            }
        }
        Err(e) => error!("Fork failed: {}", e),
    }
}

fn save_slot(id: u16, slot: &Slot) {
    let file_name = format!("slot_{}", id);
    let path = Path::new(&file_name);
    let display = path.display();
    let file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}", display, why),
        Ok(file) => BufWriter::new(file),
    };
    bincode::serialize_into(file, &slot).unwrap();
}
