use std::{sync::Arc, time::Duration};

use common::{config::CONFIG, SYNC_CMD_PING};
use parking_lot::Mutex;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    select, time,
};
use tracing::warn;

use super::Message;

#[derive(Clone)]
pub struct ForwardConnections(Arc<Mutex<Vec<flume::Sender<super::Message>>>>);

impl ForwardConnections {
    pub async fn new() -> Self {
        let fc = Self(Arc::new(Mutex::new(vec![])));
        let res = fc.clone();
        tokio::spawn(async move {
            let mut listener = Listener::new(TcpListener::bind(CONFIG.forward_addr).await.unwrap());
            loop {
                let stream = listener.accept().await.unwrap();
                let fc = fc.clone();
                tokio::spawn(async move {
                    let (tx, rx) = flume::unbounded();
                    fc.0.lock().push(tx);
                    new_connect_task(stream, rx).await.unwrap();
                });
            }
        });
        res
    }

    pub fn push_all(&self, msg: &Message) {
        self.0.lock().retain(|t| match t.send(msg.clone()) {
            Ok(_) => true,
            Err(e) => {
                warn!("{:?}", e);
                false
            }
        });
    }
}
struct Listener {
    listener: TcpListener,
}

impl Listener {
    pub fn new(listener: TcpListener) -> Self {
        Self { listener }
    }
    pub async fn accept(&mut self) -> common::Result<TcpStream> {
        let mut backoff = 1;
        loop {
            match self.listener.accept().await {
                Ok((socket, _)) => return Ok(socket),
                Err(err) => {
                    if backoff > 64 {
                        return Err(err.into());
                    }
                }
            }
            time::sleep(Duration::from_secs(backoff)).await;
            backoff *= 2;
        }
    }
}

async fn new_connect_task(
    mut stream: TcpStream,
    rx: flume::Receiver<super::Message>,
) -> common::Result<()> {
    let mut ping_buf = [0; SYNC_CMD_PING.len()];
    loop {
        select! {
            // 转发消息
            msg = rx.recv_async() => {
                let _ = stream.write_all(&bincode::serialize(&msg?)?).await;
            }
            // 处理心跳
            n = stream.read(&mut ping_buf) => {
                if n? == 0 {
                    return Err("EOF".into());
                }
                let _ = stream.write_all(&bincode::serialize(&super::Message::none())?).await;
            }
        }
    }
}
