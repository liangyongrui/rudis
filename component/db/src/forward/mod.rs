use std::time::Duration;

use arc_swap::ArcSwapOption;
use once_cell::sync::Lazy;
use tokio::time::sleep;
use tracing::info;

pub use self::message::Message;
use crate::Db;

pub mod message;

pub static FORWARD: Lazy<Forward> = Lazy::new(|| {
    info!("init forward mod");
    let f = Forward::new();
    tokio::spawn(async {
        sleep(Duration::from_secs(1)).await;
        FORWARD.run().await;
    });
    info!("init forward mod end");
    f
});

/// 转发服务的状态
pub struct Forward {
    pub tx: flume::Sender<Message>,
    rx: flume::Receiver<Message>,
    pub hdp_sender: ArcSwapOption<flume::Sender<Message>>,
    pub replica_sender: ArcSwapOption<flume::Sender<Message>>,
    pub db: ArcSwapOption<Db>,
}

impl Forward {
    fn new() -> Self {
        let (tx, rx) = flume::unbounded();
        Self {
            tx,
            rx,
            hdp_sender: ArcSwapOption::new(None),
            replica_sender: ArcSwapOption::new(None),
            db: ArcSwapOption::new(None),
        }
    }

    async fn run(&self) {
        while let Ok(msg) = self.rx.recv_async().await {
            match (
                self.hdp_sender.load().as_deref(),
                self.replica_sender.load().as_deref(),
            ) {
                (None, None) => (),
                (None, Some(s)) | (Some(s), None) => {
                    let _ = s.send(msg);
                }
                (Some(s), Some(s2)) => {
                    let _ = s.send(msg.clone());
                    let _ = s2.send(msg);
                }
            }
        }
    }
}
