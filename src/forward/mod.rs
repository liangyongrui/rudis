use fixed_vec_deque::FixedVecDeque;
use tokio::sync::mpsc;

pub use self::message::Message;

pub mod message;

/// 转发服务的状态
pub struct Forward {
    /// todo 这个大小需要调整
    buf: FixedVecDeque<[Message; 128]>,
    pub tx: mpsc::Sender<Message>,
    rx: mpsc::Receiver<Message>,
}

impl Forward {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(1024);
        Self {
            tx,
            rx,
            buf: FixedVecDeque::new(),
        }
    }

    pub fn listen(self) {
        tokio::spawn(self.run());
    }

    async fn run(mut self) {
        while let Some(msg) = self.rx.recv().await {
            *self.buf.push_back() = msg.clone();
            // todo aof, 主从同步...
        }
    }
}
