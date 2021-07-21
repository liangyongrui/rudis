use fixed_vec_deque::FixedVecDeque;
use once_cell::sync::Lazy;
use tokio::sync::mpsc;

use self::message::Message;

pub mod message;

/// 转发服务的状态
struct Forward {
    buf: FixedVecDeque<[Message; 16384]>,
}

impl Forward {
    pub fn new() -> Self {
        Self {
            buf: FixedVecDeque::new(),
        }
    }

    pub fn listen(self) -> mpsc::Sender<Message> {
        let (tx, rx) = mpsc::channel(1024);
        tokio::spawn(self.run(rx));
        tx
    }

    async fn run(mut self, mut rx: mpsc::Receiver<Message>) {
        while let Some(msg) = rx.recv().await {
            *self.buf.push_back() = msg.clone();
            // todo aof, 主从同步...
        }
    }
}
