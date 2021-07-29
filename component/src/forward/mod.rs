use fixed_queue::FixedQueue;

pub use self::message::Message;
use crate::hdp::HdpCmd;

pub mod message;

/// 转发服务的状态
pub struct Forward {
    buf: FixedQueue<Message>,
    pub tx: flume::Sender<Message>,
    rx: flume::Receiver<Message>,
    hdp_sender: Option<flume::Sender<HdpCmd>>,
}

impl Forward {
    pub fn new(hdp_sender: Option<flume::Sender<HdpCmd>>) -> Self {
        let (tx, rx) = flume::unbounded();
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
        while let Ok(msg) = self.rx.recv_async().await {
            self.buf.push(msg.clone());
            if let Some(ref hdp) = self.hdp_sender {
                let _ = hdp.send(HdpCmd::ForwardWrite(msg));
            }
            // todo aof, 主从同步...
        }
    }
}
