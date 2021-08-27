use std::time::Duration;

use connection::{
    parse::{self, Parse},
    Connection,
};
use tokio::{
    net::{TcpListener, TcpStream},
    time,
};
use tracing::error;

/// 全局状态
mod status;

struct Listener {
    listener: TcpListener,
}
impl Listener {
    async fn accept(&mut self) -> common::Result<TcpStream> {
        let mut backoff = 1;
        loop {
            match self.listener.accept().await {
                Ok((socket, _)) => return Ok(socket),
                Err(err) => {
                    if backoff > 64 {
                        return Err(err.into());
                    }
                }
            }
            time::sleep(Duration::from_secs(backoff)).await;
            backoff *= 2;
        }
    }
}

pub async fn run(listener: TcpListener) -> common::Result<()> {
    let mut server = Listener { listener };
    loop {
        let socket = server.accept().await?;
        let connection = Connection::new(socket);
        tokio::spawn(async move {
            if let Err(err) = connection_run(connection).await {
                error!(cause = ?err, "connection error");
            }
        });
    }
}

async fn connection_run(mut connection: Connection) -> common::Result<()> {
    loop {
        let maybe_frame = connection.read_frame().await?;

        let frame = match maybe_frame {
            Some(frame) => frame,
            None => return Ok(()),
        };
        let mut parse = Parse::new(frame)?;
        let command_name = parse.next_string()?.to_lowercase();
    }
}
