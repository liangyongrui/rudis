use std::time::Duration;

use connection::{
    parse::{frame::Frame, Parse},
    Connection,
};
use tokio::{
    io::AsyncWriteExt,
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
            let parse = &mut Parse::new(frame)?;
            let res = Self::apply_frame(parse);
            let res = match res {
                Ok(f) => f,
                Err(e) => Frame::OwnedError(e.to_string()),
            };
            let bytes: Vec<u8> = (&res).into();
            self.connection.stream.write_all(&bytes).await?;
        }
    }

    pub fn apply_frame<'a>(parse: &'a mut Parse<'a>) -> common::Result<Frame<'a>> {
        let command_name = parse.next_string()?.to_lowercase();
        match &command_name[..] {
            common::pd_message::cmd::SEVER_INIT => server_init_apply(parse),
            common::pd_message::cmd::SEVER_HEARTBEAT => server_heartbeat_apply(parse),
            common::pd_message::cmd::CRATE_GROUP => Ok(crate_group_apply()),
            _ => Ok(Frame::ok()),
        }
    }
}
