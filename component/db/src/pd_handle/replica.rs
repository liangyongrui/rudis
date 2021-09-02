use std::{
    io::{BufReader, Write},
    net::{SocketAddr, TcpStream},
    sync::{atomic::AtomicBool, Arc},
    time::{Duration, Instant},
};

use arc_swap::ArcSwapOption;
use common::{pd_message::LeaderInfo, SYNC_CMD};
use connection::parse::frame::Frame;
use dict::Dict;
use tokio::sync::Notify;
use tracing::error;

use crate::{forward::Message, Db, SLOT_SIZE};

struct Status {
    snapshot_lock: [AtomicBool; SLOT_SIZE],
    cmd_rx: ArcSwapOption<flume::Receiver<Message>>,
    db: Arc<Db>,
    notify: Arc<Notify>,
    leader: LeaderInfo,
}

impl Status {
    pub fn sync_all_snapshot(&mut self) -> common::Result<()> {
        for l in &self.snapshot_lock {
            l.store(true, std::sync::atomic::Ordering::Release);
        }
        self.sync_snapshot(u16::MAX as _)
    }

    /// 这里的tcp是阻塞io
    pub fn sync_snapshot(&self, slot_id: usize) -> common::Result<()> {
        self.snapshot_lock[slot_id].store(true, std::sync::atomic::Ordering::Release);
        let mut stream = TcpStream::connect(self.leader.server_addr)?;
        let req: Vec<_> = (&Frame::Array(vec![
            Frame::Bulk(b"syncsnapshot"[..].into()),
            Frame::Integer(slot_id as _),
        ]))
            .into();
        stream.write_all(&req)?;
        loop {
            let slot_id: u16 = match bincode::deserialize_from(&mut stream)? {
                Some(slot) => slot,
                None => return Ok(()),
            };
            let slot_id = slot_id as usize;
            let dict: Dict = bincode::deserialize_from(&mut stream)?;
            self.db.replace_dict(slot_id, dict);
            self.snapshot_lock[slot_id].store(false, std::sync::atomic::Ordering::Release);
        }
    }

    pub fn close_sync_cmd(& self) {
        self.cmd_rx.store(None)
    }
    pub fn sync_cmd(&mut self, leader_forward_socket: SocketAddr) -> common::Result<()> {
        let (tx, rx) = flume::unbounded();
        let mut stream = TcpStream::connect(leader_forward_socket)?;
        stream.write_all(SYNC_CMD)?;
        let stream = BufReader::new(stream);
        tokio::task::spawn_blocking(move || {
            if let Err(e) = receive_cmd(tx, stream) {
                error!("{:?}", e)
            }
        });
        self.cmd_rx.store(  Some(rx));
        tokio::spawn(self.process_cmd());
        Ok(())
    }
    async fn process_cmd(self: Arc<Self>) {
        loop {
            if let Some(rx) = &self.cmd_rx {
                if let Ok(msg) = rx.recv_async().await {
                    let slot_id = msg.slot;
                    loop {
                        self.wait_slot(slot_id).await;
                        if let std::cmp::Ordering::Greater =
                            self.db.slots[slot_id].process_forward(msg.id, msg.cmd.clone())
                        {
                            let s = self.clone();
                            tokio::task::spawn_blocking(move || s.sync_snapshot(slot_id))
                                .await
                                .unwrap()
                                .unwrap();
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

    async fn wait_slot(&self, slot_id: usize) {
        loop {
            if self.snapshot_lock[slot_id].load(std::sync::atomic::Ordering::Acquire) {
                self.notify.notified().await
            } else {
                return;
            }
        }
    }
}

fn receive_cmd(tx: flume::Sender<Message>, mut stream: BufReader<TcpStream>) -> common::Result<()> {
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
