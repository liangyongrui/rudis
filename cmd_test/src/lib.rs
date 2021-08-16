//! 测试用的一些基础函数

use core::panic;

use ::server::{server, Connection, Frame};
use tokio::{
    io::AsyncWriteExt,
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

pub async fn next_frame_eq(connection: &mut Connection, right: Frame) {
    let left = connection.read_frame().await.unwrap().unwrap();
    assert_eq!(left, right);
}

pub async fn next_array_frame_sorted_eq(connection: &mut Connection, mut right: Vec<Frame>) {
    let left = connection.read_frame().await.unwrap().unwrap();
    if let Frame::Array(mut left) = left {
        left.sort();
        right.sort();
        assert_eq!(left, right);
    } else {
        panic!("{:?}", left);
    }
}

pub async fn next_frame_in(connection: &mut Connection, rights: Vec<Frame>) {
    let left = connection.read_frame().await.unwrap().unwrap();
    for right in rights {
        if left == right {
            return;
        }
    }
    panic!("{:?}", left);
}

pub async fn start_server() -> Connection {
    let _ = tracing_subscriber::fmt::Subscriber::builder()
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
        .with_max_level(tracing::Level::DEBUG)
        .try_init();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    debug!(?addr);
    tokio::spawn(async move { server::run(listener, tokio::signal::ctrl_c()).await });
    Connection::new(TcpStream::connect(addr).await.unwrap())
}
