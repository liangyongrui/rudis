use std::{net::SocketAddr, sync::Arc};

use bytes::BytesMut;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    net::{TcpStream, ToSocketAddrs},
};

use crate::{db::Db, slot::dict::Dict, Frame};

pub struct Connection {
    db: Arc<Db>,
    buffer: BytesMut,
    stream: BufReader<TcpStream>,
}

impl Connection {
    /// 创建主从连接
    async fn new<A: ToSocketAddrs>(addr: A, db: Arc<Db>) -> Self {
        let stream = TcpStream::connect(addr).await.unwrap();
        Self {
            db,
            stream: BufReader::new(stream),
            buffer: BytesMut::with_capacity(8 * 1024),
        }
    }

    async fn send_sync_cmd(&mut self) -> Result<(), std::io::Error> {
        let f = Frame::Array(vec![Frame::Bulk(b"sync"[..].into())]);
        let b: Vec<u8> = (&f).into();
        self.stream.write_all(&b).await
    }

    async fn listen_master(&mut self) -> crate::Result<()> {
        // 读dict
        loop {
            let (slot, dict) = match self.read_dict().await? {
                Some(r) => r,
                None => break,
            };
            self.db.update_dict(slot, dict);
        }
        Ok(())
    }
    /// 读取一个 dict
    ///
    /// # WARN
    /// - 32 位平台单 dict 最多 2GB 的数据
    /// - 64 位平台单 dict 可以有 2^33GB 的数据
    async fn read_dict(&mut self) -> crate::Result<Option<(u16, Dict)>> {
        let mut slot_id_buf = [0u8; 2];
        self.stream.read_exact(&mut slot_id_buf).await?;
        let slot_id = u16::from_be_bytes(slot_id_buf);
        if slot_id == u16::MAX {
            // dict 读完了
            return Ok(None);
        }

        let mut len_buf = [0u8; 8];
        self.stream.read_exact(&mut len_buf).await?;
        let len = u64::from_be_bytes(len_buf);
        if len > isize::MAX as u64 {
            return Err("The copied data is too large.".into());
        }
        let mut dict_bincode_buf = vec![0u8; len as _];
        self.stream.read_exact(&mut dict_bincode_buf).await?;
        let slot_id = u16::from_be_bytes(slot_id_buf);
        let dict: Dict = bincode::deserialize(&dict_bincode_buf)?;
        return Ok(Some((slot_id, dict)));
    }
}

pub(crate) fn update_master(_master_addr: SocketAddr) {
    // todo!()
}
