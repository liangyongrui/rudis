use std::{
    fs::File,
    io::{BufWriter, Write},
    time::Instant,
};

use tokio::sync::mpsc;

use crate::{cmd::WriteCmd, config::CONFIG};

pub struct Aof {
    _aof_max_backlog: u64,
    rx: mpsc::Receiver<WriteCmd>,
}

impl Aof {
    pub fn start() -> Option<mpsc::Sender<WriteCmd>> {
        CONFIG.save_aof_path.as_ref()?;
        let aof_max_backlog = CONFIG.aof_max_backlog;
        if aof_max_backlog == 0 {
            return None;
        }
        let (tx, rx) = mpsc::channel(aof_max_backlog as _);
        let aof = Aof {
            _aof_max_backlog: aof_max_backlog,
            rx,
        };
        tokio::spawn(aof.listener());
        Some(tx)
    }

    async fn listener(mut self) {
        let path = CONFIG.save_aof_path.as_ref().unwrap();
        let display = path.display();
        let mut file = match File::create(path) {
            Err(why) => panic!("couldn't create {}: {}", display, why),
            Ok(file) => BufWriter::new(file),
        };
        let now = Instant::now();
        let mut update_secs = 0;
        while let Some(cmd) = self.rx.recv().await {
            file.write_all(&cmd.into_cmd_bytes()[..]).unwrap();
            let secs = (Instant::now() - now).as_secs();
            if secs > update_secs {
                update_secs = secs;
                file.flush().unwrap();
            }
        }
        file.flush().unwrap();
    }
}
