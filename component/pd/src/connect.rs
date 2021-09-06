use std::time::Duration;

use connection::{
    parse::{frame::Frame, Parse},
    Connection,
};
use tokio::{
    net::{TcpListener, TcpStream},
    time,
};

use crate::cmd::{crate_group_apply, server_heartbeat_apply, server_init_apply};

pub struct Listener {
    listener: TcpListener,
}
impl Listener {
    pub fn new(listener: TcpListener) -> Self {
        Self { listener }
    }
    pub async fn accept(&mut self) -> common::Result<TcpStream> {
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

pub struct Handle {
    connection: Connection,
}

impl Handle {
    pub fn new(connection: Connection) -> Self {
        Self { connection }
    }

    pub async fn run(&mut self) -> common::Result<()> {
        loop {
            let maybe_frame = self.connection.read_frame().await?;
            let frame = match maybe_frame {
                Some(frame) => frame,
                None => return Ok(()),
            };
            let res = Self::apply_frame(frame);
            let res = match res {
                Ok(f) => f,
                Err(e) => Frame::Error(e.to_string().as_bytes().into()),
            };
            self.connection.write_frame(&res).await?;
        }
    }

    pub fn apply_frame(frame: Frame) -> common::Result<Frame> {
        let parse = &mut Parse::new(frame)?;
        let command_name = parse.next_string()?.to_lowercase();
        match &command_name[..] {
            common::pd_message::cmd::SEVER_INIT => server_init_apply(parse),
            common::pd_message::cmd::SEVER_HEARTBEAT => server_heartbeat_apply(parse),
            common::pd_message::cmd::CRATE_GROUP => Ok(crate_group_apply()),
            _ => Ok(Frame::ok()),
        }
    }
}
