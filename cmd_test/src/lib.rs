//! 测试用的一些基础函数

use component::{server, Frame};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use tracing::{debug, Level};

trait NewCmd {
    fn cmd(self) -> Vec<u8>;
}

impl NewCmd for &str {
    fn cmd(self) -> Vec<u8> {
        let args = self
            .split_ascii_whitespace()
            .map(|t| Frame::Simple(t.to_owned()))
            .collect();
        Frame::Array(args).into()
    }
}

pub async fn write_cmd(stream: &mut TcpStream, cmd: &str) {
    stream.write_all(&cmd.cmd()).await.unwrap();
}

pub async fn read_assert_eq(stream: &mut TcpStream, right: &[u8]) {
    let mut response = vec![0; right.len()];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(right, &response);
}

pub async fn start_server() -> TcpStream {
    let _ = tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(Level::DEBUG)
        .try_init();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    debug!(?addr);
    tokio::spawn(async move { server::run(listener, tokio::signal::ctrl_c()).await });
    TcpStream::connect(addr).await.unwrap()
}
