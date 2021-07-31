use std::{
    io::{BufReader, Write},
    net::{TcpStream, ToSocketAddrs},
    sync::Arc,
    thread,
    time::Duration,
};

use tokio::sync::{broadcast, Notify};

use crate::{
    cmd::{SYNC_CMD, SYNC_CMD_PING, SYNC_SNAPSHOT},
    db::{Db, Role},
    forward,
    shutdown::Shutdown,
    slot::dict::Dict,
};
/// 发起主从复制
///
/// warn: 这个函数是阻塞的
pub fn process<A: ToSocketAddrs + Clone>(addr: A, db: Arc<Db>) -> crate::Result<()> {
    let notify = Arc::new(Notify::new());
    let res = process_cmd_forward(addr.clone(), db.clone(), notify.clone())?;
    process_snapshot(addr, db.clone())?;
    notify.notify_one();
    *db.role.lock() = Role::Replica(res);
    Ok(())
}

/// 向主节点要快照
fn process_snapshot<A: ToSocketAddrs + Clone>(addr: A, db: Arc<Db>) -> crate::Result<()> {
    let mut stream = BufReader::new(TcpStream::connect(addr)?);
    stream.get_mut().write_all(SYNC_SNAPSHOT)?;
    loop {
        let slot_id: u16 = match bincode::deserialize_from(&mut stream)? {
            Some(slot) => slot,
            None => return Ok(()),
        };
        let dict: Dict = bincode::deserialize_from(&mut stream)?;
        db.replace_dict(slot_id, dict);
    }
}

/// 同步主节点的 write cmd
fn process_cmd_forward<A: ToSocketAddrs>(
    addr: A,
    db: Arc<Db>,
    notify: Arc<Notify>,
) -> crate::Result<broadcast::Sender<()>> {
    let stream = TcpStream::connect(addr)?;
    let mut writer = stream.try_clone()?;
    writer.write_all(SYNC_CMD).unwrap();
    let (shutdown_sender, _) = broadcast::channel(1);
    let (tx, rx) = flume::unbounded();

    // 读取转发来的消息
    let mut reader = BufReader::new(stream);
    let mut shutdown = Shutdown::new(shutdown_sender.subscribe());
    tokio::task::spawn_blocking(move || {
        while !shutdown.check_shutdown() {
            let message: forward::Message = bincode::deserialize_from(&mut reader).unwrap();
            tx.send(message).unwrap();
        }
    });

    // 定时心跳
    let mut shutdown = Shutdown::new(shutdown_sender.subscribe());
    tokio::task::spawn_blocking(move || {
        while !shutdown.check_shutdown() {
            thread::sleep(Duration::from_secs(10));
            writer.write_all(SYNC_CMD_PING).unwrap();
        }
    });

    // 消费channel
    tokio::spawn(async move {
        // 等snapshot准备好
        notify.notified().await;
        while let Ok(msg) = rx.recv_async().await {
            db.process_forward(msg);
        }
    });

    Ok(shutdown_sender)
}
