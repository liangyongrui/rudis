use fixed_queue::FixedQueue;
use tokio::sync::mpsc;

pub use self::message::Message;

pub mod message;

/// 转发服务的状态
pub struct Forward {
    buf: FixedQueue<Message>,
    pub tx: mpsc::Sender<Message>,
    rx: mpsc::Receiver<Message>,
}

impl Forward {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(1024);
        Self {
            tx,
            rx,
            buf: FixedQueue::new(16384),
        }
    }

    pub fn listen(self) {
        tokio::spawn(self.run());
    }

    async fn run(mut self) {
        while let Some(msg) = self.rx.recv().await {
            self.buf.push(msg.clone());
            // todo aof, 主从同步...
        }
    }
}
