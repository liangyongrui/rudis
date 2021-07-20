use std::{
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
    time::Instant,
};

use tokio::sync::mpsc;

use crate::{cmd::WriteCmd, config::CONFIG};

pub struct Aof {
    rx: mpsc::Receiver<WriteCmd>,
    save_path: PathBuf,
}

impl Aof {
    pub fn start(create_timestamp: u64) -> Option<mpsc::Sender<WriteCmd>> {
        let aof_max_backlog = CONFIG.aof_max_backlog;
        let save_path = if let Some(ref dir) = CONFIG.save_aof_dir {
            dir.join(format!("dump_{}.aof", create_timestamp))
        } else {
            return None;
        };
        if aof_max_backlog == 0 {
            return None;
        }
        let (tx, rx) = mpsc::channel(aof_max_backlog as _);
        let aof = Aof { rx, save_path };
        tokio::spawn(aof.run());
        Some(tx)
    }

    async fn run(mut self) {
        let display = self.save_path.display();
        let mut file = match File::create(&self.save_path) {
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
