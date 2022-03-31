use std::{collections::HashMap, thread::sleep, time::Duration};

use nix::{
    sys::wait::{waitpid, WaitPidFlag},
    unistd::Pid,
};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use tracing::{error, info, warn};

static STATUS: Lazy<Mutex<HashMap<Pid, Info, ahash::RandomState>>> = Lazy::new(|| {
    tokio::task::spawn_blocking(loop_status);
    Mutex::new(HashMap::with_hasher(ahash::RandomState::default()))
});

#[derive(Debug)]
pub enum Info {
    HdpSnapshot { base_id: u64 },
    SyncSnapshot,
}

fn loop_status() {
    loop {
        sleep(Duration::from_secs(1));
        match waitpid(None, Some(WaitPidFlag::WNOHANG)) {
            Ok(nix::sys::wait::WaitStatus::Exited(pid, _)) => {
                if let Some(e) = STATUS.lock().remove(&pid) {
                    info!("{:?} exited: {}", e, pid);
                } else {
                    error!("unknown pid: {}", pid);
                }
            }
            Ok(nix::sys::wait::WaitStatus::StillAlive) => (),
            Ok(other_status) => warn!(?other_status),
            Err(e) => error!(?e),
        }
    }
}

pub fn add(pid: Pid, info: Info) {
    STATUS.lock().insert(pid, info);
}
