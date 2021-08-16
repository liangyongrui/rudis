use std::path::Path;

use common::config::CONFIG;
use tokio::{
    fs::File,
    io::{AsyncWriteExt, BufWriter},
};
use tracing::{debug, error};

use crate::{forward::Message, hdp::snapshot};

/// 每个 slot 都有一个aof_status
pub struct Status {
    pub slot_id: u16,
    pub snapshot_next_id: u64,
    pub file: BufWriter<File>,
    pub next_id: u64,
    pub count: u64,
}

impl Status {
    pub fn new(save_hdp_dir: &Path, snapshot_next_id: u64, slot_id: u16) -> common::Result<Self> {
        let save_path = save_hdp_dir.join(format!("{}/dump_{}.aof", snapshot_next_id, slot_id));
        let display = &save_path.display();
        match std::fs::File::create(&save_path) {
            Err(why) => Err(format!("couldn't create {}: {}", display, why).into()),
            Ok(file) => {
                let file = File::from_std(file);
                Ok(Self {
                    slot_id,
                    snapshot_next_id,
                    count: 0,
                    file: BufWriter::new(file),
                    next_id: snapshot_next_id,
                })
            }
        }
    }

    pub async fn flush(&mut self) -> common::Result<()> {
        self.file.flush().await.map_err(|t| t.into())
    }

    /// 返回是否需要更新 snapshot
    pub async fn write(&mut self, message: &Message) -> bool {
        match self.next_id.cmp(&message.id) {
            std::cmp::Ordering::Less => unreachable!(),
            // 写入文件
            std::cmp::Ordering::Equal => {
                let bc = bincode::serialize(message).unwrap();
                match self.file.write_all(&bc).await {
                    Ok(_) => debug!(?bc),
                    Err(e) => error!(?e),
                }
            }
            // 忽略不处理
            std::cmp::Ordering::Greater => (),
        };
        self.next_id += 1;
        self.count += 1;
        CONFIG.hdp.aof_count > 0
            && self.count >= CONFIG.hdp.aof_count
            && snapshot::IN_PROGRESS
                .compare_exchange(
                    false,
                    true,
                    std::sync::atomic::Ordering::SeqCst,
                    std::sync::atomic::Ordering::SeqCst,
                )
                .is_ok()
    }
}
