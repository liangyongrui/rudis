use tokio::sync::mpsc;

use crate::{cmd::WriteCmd, config::CONFIG};

pub struct Aof {
    aof_max_backlog: u64,
    rx: mpsc::Receiver<WriteCmd>,
}

impl Aof {
    pub fn start() -> Option<mpsc::Sender<WriteCmd>> {
        let aof_max_backlog = CONFIG.aof_max_backlog;
        if aof_max_backlog == 0 {
            return None;
        }
        let (tx, rx) = mpsc::channel(aof_max_backlog as _);
        let aof = Aof {
            aof_max_backlog,
            rx,
        };
        tokio::spawn(aof.listener());
        Some(tx)
    }

    async fn listener(mut self) {
        while let Some(cmd) = self.rx.recv().await {}
    }
}
