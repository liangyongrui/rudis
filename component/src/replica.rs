use std::net::SocketAddr;

use bytes::BytesMut;
use tokio::net::{TcpStream, ToSocketAddrs};

pub struct Connection {
    buffer: BytesMut,
    stream: TcpStream,
}

impl Connection {
    /// 创建主从连接
    pub async fn new<A: ToSocketAddrs>(addr: A) -> Self {
        let stream = TcpStream::connect(addr).await.unwrap();
        Self {
            stream,
            buffer: BytesMut::with_capacity(4 * 1024),
        }
    }
}

pub(crate) fn update_master(_master_addr: SocketAddr) {
    // todo!()
}
