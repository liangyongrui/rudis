//! 测试用的一些基础函数

use component::{server, Frame};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use tracing::debug;
trait NewCmd {
    fn cmd(self) -> Vec<u8>;
}

pub async fn write_cmd(stream: &mut TcpStream, cmd: Vec<&str>) {
    debug!(?cmd);
    let args = cmd
        .into_iter()
        .map(|t| Frame::Bulk(t.as_bytes().into()))
        .collect();
    let cmd: Vec<u8> = (&Frame::Array(args)).into();
    stream.write_all(&cmd).await.unwrap();
    stream.flush().await.unwrap();
}

pub async fn read_assert_eq(stream: &mut TcpStream, right: &[u8]) {
    let mut response = vec![0; right.len()];
    stream.read_exact(&mut response).await.unwrap();
    debug!("{:?}", std::str::from_utf8(&response));
    assert_eq!(&response, right);
}

pub async fn start_server() -> TcpStream {
    let _ = tracing_subscriber::fmt::Subscriber::builder()
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
        .with_max_level(tracing::Level::DEBUG)
        .try_init();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    debug!(?addr);
    tokio::spawn(async move { server::run(listener, tokio::signal::ctrl_c()).await });
    TcpStream::connect(addr).await.unwrap()
}
