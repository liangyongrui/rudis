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
use tokio::net::TcpStream;

use crate::Db;

pub async fn run(db: Arc<Db>, pd: Pd) -> common::Result<()> {
    let connection = Connection::new(TcpStream::connect(pd.addr).await?);
    let handle = PdHandle {
        connection,
        pd,
        db,
        latest_status: ServerStatus::default(),
    };
    handle.init().await
}
struct PdHandle {
    connection: Connection,
    pd: Pd,
    db: Arc<Db>,
    latest_status: ServerStatus,
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

    async fn read_bytes(&mut self) -> common::Result<Arc<[u8]>> {
        let frame = match self.connection.read_frame().await? {
            Some(f) => f,
            None => todo!(),
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
                self.update_status(status)?;
            }
        }
    }

    fn update_status(&self, status: ServerStatus) -> common::Result<()> {
        match (self.latest_status.role, status.role) {
            (_, ServerRole::Follower)
                if self.latest_status.current_leader != status.current_leader =>
            {
                // update leader
            }
            (ServerRole::Follower, ServerRole::Leader) => {
                // 自己变成leader
            }
            _ => {
                // do nothing
            }
        }
        Ok(())
    }
}
