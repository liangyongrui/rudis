use std::process::exit;

use common::SLOT_SIZE;
use connection::parse::Parse;
use db::child_process;
use nix::unistd::ForkResult;
use tokio::net::TcpStream;
use tracing::{error, info};

use crate::Handler;

#[derive(Debug)]
pub struct SyncSnapshot {
    slot_id: usize,
}

impl SyncSnapshot {
    pub fn parse_frames(parse: &Parse) -> common::Result<Self> {
        Ok(Self {
            slot_id: parse.next_int()? as usize,
        })
    }

    #[tracing::instrument(skip(handler))]
    pub fn apply(self, handler: Handler) {
        // 先拿到需要传输的数据，避免死锁
        let slots = if self.slot_id == SLOT_SIZE as usize {
            // 全部slot
            handler
                .db
                .slots
                .iter()
                .map(|t| (t.slot_id, t.share_status.read()))
                .collect()
        } else {
            // 指定的slot
            vec![(
                self.slot_id,
                handler.db.get_slot_by_id(self.slot_id).share_status.read(),
            )]
        };

        match unsafe { nix::unistd::fork() } {
            Ok(ForkResult::Parent { child }) => {
                info!(
                    "Continuing execution in parent process, new child has pid: {}",
                    child
                );
                child_process::add(child, child_process::Info::SyncSnapshot);
            }
            Ok(ForkResult::Child) => {
                // todo
                // After fork, the child process will inherit the resources
                // of the parent process, e.g. fd(socket or flock) etc.
                // should close the resources not used by the child process, so that if the
                // parent restarts it can bind/lock despite the child possibly still running.
                let run = |stream: TcpStream| -> common::Result<()> {
                    let mut stream = stream.into_std()?;
                    for (id, s) in slots {
                        if let Some(s) = &*s {
                            bincode::serialize_into(&mut stream, &Some(id))?;
                            bincode::serialize_into(&mut stream, &s.dict)?;
                        }
                    }
                    let end: Option<u16> = Option::None;
                    bincode::serialize_into(&mut stream, &end)?;
                    Ok(())
                };
                match run(handler.connection.stream) {
                    Ok(_) => {}
                    Err(e) => error!("send snapshot error: {:?}", e),
                }
                exit(0);
            }
            Err(e) => error!("Fork failed: {}", e),
        }
    }
}
