#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::shadow_unrelated)]
#![allow(clippy::doc_markdown)]
#![allow(unstable_name_collisions)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::let_underscore_drop)]

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

    /// Read a frame from connection.
    /// Returning `None` means that the connection has ended and there are no unprocessed bytes.
    ///
    /// # Errors
    /// 1. parse failed
    /// 1. connect end
    /// 1. other io error
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

    /// Read a frame from the buffer.
    ///
    /// # Errors
    /// parse failed
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
