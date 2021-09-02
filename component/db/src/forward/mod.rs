mod connection;

use self::connection::ForwardConnections;
pub use self::message::Message;

pub mod message;

/// 转发服务的状态
pub struct Forward {
    pub tx: flume::Sender<Message>,
    rx: flume::Receiver<Message>,
}

impl Forward {
    pub fn new() -> Self {
        let (tx, rx) = flume::unbounded();
        Self { tx, rx }
    }
    pub fn listen(self) {
        tokio::spawn(self.run());
    }
    async fn run(self) {
        let forward_connection = ForwardConnections::new().await;
        while let Ok(msg) = self.rx.recv_async().await {
            forward_connection.push_all(&msg);
        }
    }
}
