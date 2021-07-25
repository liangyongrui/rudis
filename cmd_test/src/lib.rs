//! 测试用的一些基础函数

use component::server;
use tokio::net::{TcpListener, TcpStream};
use tracing::Level;

pub fn new_cmd() {}

pub async fn start_server() -> TcpStream {
    let _ = tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(Level::DEBUG)
        .try_init();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move { server::run(listener, tokio::signal::ctrl_c()).await });
    TcpStream::connect(addr).await.unwrap()
}
