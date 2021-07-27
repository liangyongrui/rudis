use fixed_queue::FixedQueue;
use tokio::sync::mpsc;

pub use self::message::Message;
use crate::hdp::HdpCmd;

pub mod message;

/// 转发服务的状态
pub struct Forward {
    buf: FixedQueue<Message>,
    pub tx: mpsc::Sender<Message>,
    rx: mpsc::Receiver<Message>,
    hdp_sender: Option<mpsc::Sender<HdpCmd>>,
}

impl Forward {
    pub fn new(hdp_sender: Option<mpsc::Sender<HdpCmd>>) -> Self {
        let (tx, rx) = mpsc::channel(1024);
        Self {
            tx,
            rx,
            buf: FixedQueue::new(16384),
            hdp_sender,
        }
    }

    pub fn listen(self) {
        tokio::spawn(self.run());
    }

    async fn run(mut self) {
        while let Some(msg) = self.rx.recv().await {
            self.buf.push(msg.clone());
            if let Some(ref hdp) = self.hdp_sender {
                let _ = hdp.send(HdpCmd::ForwardWrite(msg)).await;
            }
            // todo aof, 主从同步...
        }
    }
}
