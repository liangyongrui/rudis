mod replica;

use std::{sync::Arc, time::Duration};

use common::{
    config::{Pd, CONFIG},
    pd_message::{
        cmd::{SEVER_HEARTBEAT, SEVER_INIT},
        ServerInit, ServerRole, ServerStatus,
    },
};
use connection::{parse::frame::Frame, Connection};
use keys::Key;
use tokio::net::TcpStream;
use tracing::error;

use crate::Db;

pub async fn run(db: Arc<Db>, pd: Pd) -> common::Result<()> {
    let connection = Connection::new(TcpStream::connect(pd.addr).await?);
    let handle = PdHandle {
        connection,
        pd,
        latest_status: ServerStatus::default(),
        replica_task: replica::Task::new(db),
    };
    handle.init().await
}
struct PdHandle {
    connection: Connection,
    pd: Pd,
    latest_status: ServerStatus,
    replica_task: replica::Task,
}

impl PdHandle {
    async fn init(mut self) -> common::Result<()> {
        self.connection
            .write_frame(&Frame::Array(vec![
                Frame::Bulk(SEVER_INIT.as_bytes().into()),
                Frame::Bulk(
                    bincode::serialize(&ServerInit {
                        group_id: self.pd.group_id,
                        server_addr: CONFIG.server_addr,
                        forward_addr: CONFIG.forward_addr,
                    })?
                    .into(),
                ),
            ]))
            .await?;
        let status: ServerStatus = bincode::deserialize(&self.read_bytes().await?)?;
        let res = self.update_status(status);
        tokio::spawn(self.heartbeat());
        res
    }

    async fn read_bytes(&mut self) -> common::Result<Key> {
        let frame = match self.connection.read_frame().await? {
            Some(f) => f,
            None => return Err("EOF".into()),
        };
        match frame {
            Frame::Bulk(b) => Ok(b),
            e => Err(format!("{:?}", e).into()),
        }
    }

    async fn heartbeat(mut self) -> common::Result<()> {
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        loop {
            interval.tick().await;
            self.connection
                .write_frame(&Frame::Array(vec![
                    Frame::Bulk(SEVER_HEARTBEAT.as_bytes().into()),
                    Frame::Bulk(bincode::serialize(&self.latest_status)?.into()),
                ]))
                .await?;
            let status: Option<ServerStatus> = bincode::deserialize(&self.read_bytes().await?)?;
            if let Some(status) = status {
                if let Err(e) = self.update_status(status) {
                    error!("{:?}", e);
                }
            }
        }
    }

    fn update_status(&mut self, status: ServerStatus) -> common::Result<()> {
        match (self.latest_status.role, status.role) {
            (_, ServerRole::Follower)
                if self.latest_status.current_leader != status.current_leader =>
            {
                // update leader
                self.replica_task.update_leader(status.current_leader);
                self.replica_task.sync_all_snapshot()?;
                self.replica_task.sync_cmd()?;
                self.latest_status = status;
            }
            (ServerRole::Follower, ServerRole::Leader) => {
                // 自己变成leader
                self.replica_task.close_sync_cmd();
                self.latest_status = status;
            }
            _ => {
                // do nothing
            }
        }
        Ok(())
    }
}
