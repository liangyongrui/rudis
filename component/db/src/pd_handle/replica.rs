use std::{
    io::{BufReader, Write},
    net::TcpStream,
    sync::{atomic::AtomicBool, Arc},
    time::{Duration, Instant},
};

use arc_swap::ArcSwapOption;
use common::{connection::parse::frame::Frame, pd_message::LeaderInfo, SYNC_CMD};
use dict::MemDict;
use parking_lot::Mutex;
use tokio::sync::Notify;
use tracing::{error, warn};

use crate::{forward::Message, Db, SLOT_SIZE};

#[derive(Clone)]
pub struct Task {
    inner: Arc<Inner>,
}

struct Inner {
    snapshot_syncing: AtomicBool,
    snapshot_lock: Vec<AtomicBool>,
    cmd_rx: ArcSwapOption<flume::Receiver<Message>>,
    db: Arc<Db>,
    notify_lock_update: Notify,
    leader: Mutex<Option<LeaderInfo>>,
}

impl Inner {
    async fn wait_slot(&self, slot_id: usize) {
        loop {
            if self.snapshot_lock[slot_id].load(std::sync::atomic::Ordering::Acquire) {
                self.notify_lock_update.notified().await;
            } else {
                break;
            }
        }
    }

    fn sync_snapshot(self: Arc<Self>, slot_id: usize) -> common::Result<()> {
        self.snapshot_syncing
            .compare_exchange(
                false,
                true,
                std::sync::atomic::Ordering::Acquire,
                std::sync::atomic::Ordering::Relaxed,
            )
            .map_err(|_| "syncing")?;
        self.lock_slot(slot_id);

        tokio::task::spawn_blocking(move || {
            if let Err(e) = self.sync_snapshot_without_lock(slot_id) {
                error!("sync_snapshot error: {:?}", e);
            }
        });

        Ok(())
    }

    fn sync_snapshot_without_lock(&self, slot_id: usize) -> common::Result<()> {
        let mut stream =
            TcpStream::connect(self.leader.lock().ok_or("leader not exists")?.server_addr)?;
        let req: Vec<_> = (&Frame::Array(vec![
            Frame::Bulk(b"syncsnapshot"[..].into()),
            Frame::Integer(slot_id as _),
        ]))
            .into();
        stream.write_all(&req)?;
        while let Some(slot_id) = bincode::deserialize_from::<_, Option<u16>>(&mut stream)? {
            let slot_id = slot_id as usize;
            let dict: MemDict = bincode::deserialize_from(&mut stream)?;
            self.db.replace_dict(slot_id, dict);
            self.snapshot_lock[slot_id].store(false, std::sync::atomic::Ordering::Release);
        }
        self.snapshot_syncing
            .store(false, std::sync::atomic::Ordering::Release);
        self.notify_lock_update.notify_one();
        Ok(())
    }

    fn lock_slot(&self, slot_id: usize) {
        if slot_id != SLOT_SIZE {}
        for l in &self.snapshot_lock {
            l.store(true, std::sync::atomic::Ordering::Relaxed);
        }
    }

    async fn process_cmd(self: Arc<Self>) {
        loop {
            if let Some(rx) = self.cmd_rx.load().as_ref() {
                if let Ok(msg) = rx.recv_async().await {
                    let slot_id = msg.slot;
                    loop {
                        self.wait_slot(slot_id).await;
                        if std::cmp::Ordering::Greater
                            == self.db.slots[slot_id].process_forward(msg.id, msg.cmd.clone())
                        {
                            if let Err(e) = self.clone().sync_snapshot(slot_id) {
                                warn!("process_cmd: {:?}", e); // 别的同步正在进行, 重试几次
                                tokio::time::sleep(Duration::from_secs(1)).await;
                            }
                        } else {
                            break;
                        }
                    }
                }
            } else {
                return;
            }
        }
    }

    fn sync_cmd(self: Arc<Self>) -> common::Result<()> {
        let (tx, rx) = flume::unbounded();
        let mut stream =
            TcpStream::connect(self.leader.lock().ok_or("leader is none")?.forward_addr)?;
        stream.write_all(SYNC_CMD)?;
        let stream = BufReader::new(stream);
        tokio::task::spawn_blocking(move || {
            if let Err(e) = receive_cmd(&tx, stream) {
                error!("sync_cmd error: {:?}", e);
            }
        });
        self.cmd_rx.store(Some(Arc::new(rx)));
        tokio::spawn(self.process_cmd());
        Ok(())
    }
}

impl Task {
    pub fn new(db: Arc<Db>) -> Self {
        let mut snapshot_lock = Vec::with_capacity(SLOT_SIZE);
        for _ in 0..SLOT_SIZE {
            snapshot_lock.push(AtomicBool::new(false));
        }
        Self {
            inner: Arc::new(Inner {
                snapshot_lock,
                cmd_rx: ArcSwapOption::new(None),
                db,
                notify_lock_update: Notify::new(),
                leader: Mutex::new(None),
                snapshot_syncing: AtomicBool::new(false),
            }),
        }
    }

    pub fn sync_all_snapshot(&self) -> common::Result<()> {
        self.inner.clone().sync_snapshot(SLOT_SIZE as _)
    }
    pub fn update_leader(&self, leader: Option<LeaderInfo>) {
        *self.inner.leader.lock() = leader;
    }

    pub fn close_sync_cmd(&self) {
        self.inner.cmd_rx.store(None);
    }
    pub fn sync_cmd(&self) -> common::Result<()> {
        self.inner.clone().sync_cmd()
    }
}

fn receive_cmd(
    tx: &flume::Sender<Message>,
    mut stream: BufReader<TcpStream>,
) -> common::Result<()> {
    let mut last_heartbeat = Instant::now();
    let duration = Duration::from_secs(1);
    loop {
        let now = Instant::now();
        if now - last_heartbeat >= duration {
            last_heartbeat = now;
            stream.get_mut().write_all(b"+PING\r\n")?;
        }
        let message: Message = bincode::deserialize_from(&mut stream)?;
        tx.send(message)?;
    }
}
