use std::{collections::HashMap, thread::sleep, time::Duration};

use nix::{
    sys::wait::{waitpid, WaitPidFlag},
    unistd::Pid,
};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use tracing::{error, warn};

static STATUS: Lazy<Mutex<HashMap<Pid, ChildProcessInfo>>> = Lazy::new(|| {
    tokio::task::spawn_blocking(loop_status);
    Mutex::new(HashMap::new())
});

pub enum ChildProcessInfo {
    Snapshot { base_id: u64 },
}

async fn loop_status() {
    loop {
        sleep(Duration::from_secs(1));
        match waitpid(None, Some(WaitPidFlag::WNOHANG)) {
            Ok(nix::sys::wait::WaitStatus::Exited(pid, _)) => match STATUS.lock().remove(&pid) {
                Some(ChildProcessInfo::Snapshot { base_id }) => finsh_snapshot(pid, base_id),
                None => error!("unknown pid: {}", pid),
            },
            Ok(nix::sys::wait::WaitStatus::StillAlive) => (),
            Ok(other_status) => warn!(?other_status),
            Err(e) => error!(?e),
        }
    }
}

pub fn add(pid: Pid, info: ChildProcessInfo) {
    STATUS.lock().insert(pid, info);
}

fn finsh_snapshot(pid: Pid, base_id: u64) {
    let _ = crate::hdp::snapshot::IN_PROGRESS.compare_exchange(
        true,
        false,
        std::sync::atomic::Ordering::SeqCst,
        std::sync::atomic::Ordering::SeqCst,
    );
    // todo 主从复制
}
