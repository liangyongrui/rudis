use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::slot::cmd::WriteCmd;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    id: u64,
    slot: u16,
    cmd: WriteCmd,
}

impl Default for Message {
    fn default() -> Self {
        Self {
            id: 0,
            slot: 0,
            cmd: WriteCmd::None,
        }
    }
}

impl Message {
    /// stream 编码
    async fn stream_encode<W: AsyncWrite + Unpin>(
        self,
        w: &mut W,
    ) -> Result<(), Box<bincode::ErrorKind>> {
        let bc = bincode::serialize(&self)?;
        w.write_all(&(bc.len() as u32).to_be_bytes()).await?;
        w.write_all(&bc).await?;
        w.flush().await?;
        Ok(())
    }

    /// stream 解码
    async fn stream_decode<R: AsyncRead + Unpin>(
        self,
        r: &mut R,
    ) -> Result<Message, Box<bincode::ErrorKind>> {
        let mut len = [0; 4];
        r.read_exact(&mut len).await?;
        let len = u32::from_be_bytes(len) as usize;
        let mut buf = vec![0; len];
        r.read_exact(&mut buf).await?;
        bincode::deserialize(&buf)
    }
}
