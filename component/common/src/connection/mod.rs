#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::shadow_unrelated)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::must_use_candidate)]

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
    advance: usize,
}

impl Connection {
    /// Create a new `Connection`, backed by `socket`.
    pub fn new(socket: TcpStream) -> Connection {
        Connection {
            stream: socket,
            read_buffer: BytesMut::with_capacity(8 * 1024),
            advance: 0,
        }
    }

    /// todo
    /// Write a single `Frame` value to the underlying stream.
    #[inline]
    #[tracing::instrument(skip(self), level = "debug")]
    pub async fn write_frame<'a>(&mut self, frame: &'a Frame<'a>) -> io::Result<()> {
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
    pub async fn read_frame(&mut self) -> crate::Result<Option<Frame<'_>>> {
        let advance = self.advance;
        if advance != 0 {
            self.read_buffer.advance(advance);
            self.advance = 0;
        }
        loop {
            // 这个 unsafe 不知道咋办
            // 可能需要等 https://github.com/rust-lang/polonius
            if let Some((advance, frame)) =
                parse::frame::parse(unsafe { &*(&mut self.read_buffer as *mut BytesMut) })?
            {
                self.advance = advance;
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
}
