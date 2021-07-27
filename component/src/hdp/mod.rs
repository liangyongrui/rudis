//! 这个模块用命令模式来驱动
//! 性能不是很敏感，单线程执行

use std::{borrow::Borrow, collections::HashMap, path::PathBuf, sync::Arc, time::Duration};

use tokio::{sync::mpsc, time};
use tracing::error;

use self::aof::AofStatus;
use crate::{config::CONFIG, db::Db, forward::Message};

mod aof;
mod snapshot;

pub enum HdpCmd {
    ForwardWrite(Message),
}

pub struct HdpStatus {
    pub tx: mpsc::Sender<HdpCmd>,
    rx: mpsc::Receiver<HdpCmd>,
    pub aof_status_map: HashMap<u16, AofStatus>,
    pub save_hdp_dir: PathBuf,
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
            save_hdp_dir,
        })
    }

    /// 执行
    pub async fn process(mut self, db: Arc<Db>) {
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
                    match cmd {
                        HdpCmd::ForwardWrite(msg) => self.process_forward_write(msg, db.borrow()).await,
                    }
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

    async fn process_forward_write(&mut self, msg: Message, db: &Db) {
        match self.aof_status_map.entry(msg.slot) {
            std::collections::hash_map::Entry::Occupied(mut e) => {
                if e.get_mut().write(&msg).await {
                    snapshot::process(self, msg.slot, db)
                }
            }
            std::collections::hash_map::Entry::Vacant(e) => {
                // 第一次 还没有snapshot 从这里创建 aofStatus
                match AofStatus::new(&self.save_hdp_dir, 0, msg.slot) {
                    Ok(status) => {
                        if e.insert(status).write(&msg).await {
                            snapshot::process(self, msg.slot, db)
                        }
                    }
                    Err(err) => {
                        error!(?err);
                    }
                }
            }
        }
    }
}
