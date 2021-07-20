use bytes::{Buf, BytesMut};
use tokio::io::{AsyncRead, AsyncReadExt, BufReader};
use tracing::error;

use crate::{parse, Frame};
#[derive(Debug)]
pub struct Reader<R: AsyncRead + Sized + Unpin> {
    stream: BufReader<R>,
    buffer: BytesMut,
}

impl<R: AsyncRead + Sized + Unpin> Reader<R> {
    pub fn new(r: R) -> Self {
        Self {
            stream: BufReader::new(r),
            buffer: BytesMut::with_capacity(4 * 1024),
        }
    }
    pub async fn read_frame(&mut self) -> crate::Result<Option<Frame>> {
        loop {
            // Attempt to parse a frame from the buffered data. If enough data
            // has been buffered, the frame is returned.
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }

            // There is not enough buffered data to read a frame. Attempt to
            // read more data from the socket.
            //
            // On success, the number of bytes is returned. `0` indicates "end
            // of stream".
            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                // The remote closed the connection. For this to be a clean
                // shutdown, there should be no data in the read buffer. If
                // there is, this means that the peer closed the socket while
                // sending a frame.
                return if self.buffer.is_empty() {
                    Ok(None)
                } else {
                    Err("connection reset by peer".into())
                };
            }
        }
    }

    fn parse_frame(&mut self) -> crate::Result<Option<Frame>> {
        let old_len = self.buffer.len();
        match parse::parse(&self.buffer[..]) {
            Ok((left, frame)) => {
                let len = old_len - left.len();
                self.buffer.advance(len);
                Ok(Some(frame))
            }
            Err(nom::Err::Incomplete(_)) => Ok(None),
            Err(e) => {
                error!(?e);
                Err("parse failed".into())
            }
        }
    }
}
