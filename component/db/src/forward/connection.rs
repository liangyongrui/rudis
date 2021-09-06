use std::{sync::Arc, time::Duration};

use common::{config::CONFIG, SYNC_CMD_PING};
use parking_lot::Mutex;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    select, time,
};
use tracing::{error, warn};

use super::Message;

#[derive(Clone)]
pub struct ForwardConnections(Arc<Mutex<Vec<flume::Sender<super::Message>>>>);

impl ForwardConnections {
    pub async fn new() -> common::Result<Self> {
        let fc = Self(Arc::new(Mutex::new(vec![])));
        let res = fc.clone();
        let mut listener = Listener::new(TcpListener::bind(CONFIG.forward_addr).await?);
        tokio::spawn(async move {
            loop {
                let stream = match listener.accept().await {
                    Ok(s) => s,
                    Err(e) => {
                        error!("forward connect accept fail: {:?}", e);
                        continue;
                    }
                };
                let fc = fc.clone();
                tokio::spawn(async move {
                    let (tx, rx) = flume::unbounded();
                    fc.0.lock().push(tx);
                    if let Err(e) = run_connect_task(stream, rx).await {
                        error!("forward fail: {:?}", e);
                    }
                });
            }
        });
        Ok(res)
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

async fn run_connect_task(
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
