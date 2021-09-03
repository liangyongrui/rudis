pub mod parse;

use std::io;

use bytes::{Buf, BytesMut};
use parse::frame::Frame;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

#[derive(Debug)]
pub struct Connection {
    pub stream: TcpStream,
    read_buffer: BytesMut,
}

impl Connection {
    /// Create a new `Connection`, backed by `socket`.
    pub fn new(socket: TcpStream) -> Connection {
        Connection {
            stream: socket,
            read_buffer: BytesMut::with_capacity(8 * 1024),
        }
    }

    /// Write a single `Frame` value to the underlying stream.
    #[inline]
    #[tracing::instrument(skip(self), level = "debug")]
    pub async fn write_frame(&mut self, frame: &Frame) -> io::Result<()> {
        let bytes: Vec<u8> = frame.into();
        self.stream.write_all(&bytes).await
    }

    pub async fn read_frame(&mut self) -> common::Result<Option<Frame>> {
        loop {
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }

            if 0 == self.stream.read_buf(&mut self.read_buffer).await? {
                return if self.read_buffer.is_empty() {
                    Ok(None)
                } else {
                    Err("connection reset by peer".into())
                };
            }
        }
    }

    #[inline]
    fn parse_frame(&mut self) -> common::Result<Option<Frame>> {
        let old_len = self.read_buffer.len();
        match parse::parse(self.read_buffer.as_ref()) {
            Ok((left, frame)) => {
                let len = old_len - left.len();
                self.read_buffer.advance(len);
                Ok(Some(frame))
            }
            Err(nom::Err::Incomplete(_)) => Ok(None),
            Err(e) => Err(format!("parse failed, {:?}", e).into()),
        }
    }
}