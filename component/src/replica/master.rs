use std::{borrow::Borrow, process::exit, sync::Arc};

use futures::prelude::*;
use nix::unistd::{fork, ForkResult};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    select,
};
use tokio_util::codec::{FramedWrite, LengthDelimitedCodec};
use tracing::{error, info, warn};

use crate::{
    child_process,
    cmd::{OK_FRAME, SYNC_CMD_PING},
    forward::{Message, FORWARD},
    server::Handler,
    slot::{cmd::WriteCmd, dict::Dict},
};

pub async fn process_sync_cmd(mut stream: TcpStream) -> crate::Result<()> {
    let (tx, rx) = flume::unbounded();
    FORWARD.replica_sender.store(Some(Arc::new(tx.clone())));
    stream.write_all(OK_FRAME).await?;
    tokio::spawn(async move {
        let (mut r, w) = stream.split();
        let length_delimited = FramedWrite::new(w, LengthDelimitedCodec::new());
        let mut serialized = tokio_serde::SymmetricallyFramed::new(
            length_delimited,
            tokio_serde::formats::Bincode::<Message, Message>::default(),
        );
        let mut ping_buf = [0; SYNC_CMD_PING.len()];
        loop {
            select! {
                // 转发消息
                msg = rx.recv_async() => {
                    match msg {
                        Ok(msg) => { let _ = serialized.send(msg).await; },
                        Err(e) => {
                            warn!(?e);
                            return;
                        },
                    }
                }
                // 处理心跳
                _ = r.read_exact(&mut ping_buf) => {
                    let _ = tx.send(Message {
                        id: 0,
                        slot: 0,
                        cmd: WriteCmd::None,
                    });
                }
            }
        }
    });

    Ok(())
}

pub fn process_snapshot(handler: Handler) -> crate::Result<()> {
    let mut stream = handler.connection.stream.into_std()?;

    // fork 子进程做snapshot，不需要持有锁
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child }) => {
            child_process::add(child, child_process::Info::SyncSnapshot);
            info!(
                "Continuing execution in parent process, new child has pid: {}",
                child
            );
        }
        Ok(ForkResult::Child) => {
            for (id, slot) in &handler.db.slots {
                let _ = bincode::serialize_into(&mut stream, &Some(id));
                let read = slot.dict.read();
                let dict: &Dict = read.borrow();
                let _ = bincode::serialize_into(&mut stream, dict);
            }
            let end: Option<u16> = Option::None;
            let _ = bincode::serialize_into(&mut stream, &end);
            exit(0);
        }
        Err(e) => error!("Fork failed: {}", e),
    }

    Ok(())
}
