mod connection;

use std::process::exit;

use tracing::error;

use self::connection::ForwardConnections;
pub use self::message::Message;

pub mod message;

/// Forward
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
        let forward_connection = match ForwardConnections::new().await {
            Ok(f) => f,
            Err(e) => {
                error!("ForwardConnections new error: {:?}", e);
                // failed to activate
                exit(-1);
            }
        };
        while let Ok(msg) = self.rx.recv_async().await {
            forward_connection.push_all(&msg);
        }
    }
}
