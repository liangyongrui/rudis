//! 这个模块用命令模式来驱动
//! 性能不是很敏感，单线程执行

use std::{
    collections::HashMap,
    path::PathBuf,
    time::{Duration, Instant},
};

use tokio::{sync::mpsc, time};
use tracing::error;

use self::{aof::AofStatus, snapshot::SnapshotStatus};
use crate::{config::CONFIG, forward::Message};

mod aof;
mod snapshot;

pub enum HdpCmd {
    ForwardWrite(Message),
}

pub struct HdpStatus {
    pub tx: mpsc::Sender<HdpCmd>,
    rx: mpsc::Receiver<HdpCmd>,
    aof_status_map: HashMap<u16, AofStatus>,
    snapshot_status: HashMap<u16, SnapshotStatus>,
    save_hdp_dir: PathBuf,
}

impl HdpStatus {
    pub async fn new() -> Option<Self> {
        let save_hdp_dir = match CONFIG.hdp.save_hdp_dir {
            Some(ref s) => s.clone(),
            None => return None,
        };
        let (tx, rx) = mpsc::channel(1024);
        Some(Self {
            tx,
            rx,
            aof_status_map: HashMap::new(),
            snapshot_status: HashMap::new(),
            save_hdp_dir,
        })
    }

    /// 执行
    async fn process(mut self) {
        let mut aof_flush_interval = time::interval(Duration::from_secs(1));
        loop {
            tokio::select! {
                _ = aof_flush_interval.tick() => {
                    self.flush_all_aof().await;
                }
                cmd = self.rx.recv() => {
                    let cmd = match cmd {
                        Some(cmd) => cmd,
                        _ => break
                    };
                    self.process_cmd(cmd).await;
                    // todo 执行cmd
                }
            }
        }
    }

    async fn flush_all_aof(&mut self) {
        for s in self.aof_status_map.values_mut() {
            match s.flush().await {
                Ok(_) => (),
                Err(e) => error!(?e),
            }
        }
    }

    async fn process_cmd(&mut self, cmd: HdpCmd) {
        async fn write_aof(aof_status: &mut AofStatus, msg: &Message) {
            match aof_status.write(msg).await {
                Ok(_) => (),
                Err(e) => error!(?e),
            }
        }

        match cmd {
            HdpCmd::ForwardWrite(msg) => match self.aof_status_map.entry(msg.slot) {
                std::collections::hash_map::Entry::Occupied(mut e) => {
                    write_aof(e.get_mut(), &msg).await
                }
                std::collections::hash_map::Entry::Vacant(e) => {
                    match AofStatus::new(&self.save_hdp_dir).await {
                        Ok(status) => write_aof(e.insert(status), &msg).await,
                        Err(err) => {
                            error!(?err);
                        }
                    }
                }
            },
        }
    }
}
