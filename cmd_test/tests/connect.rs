//! 主要测试 连接过程

use std::net::SocketAddr;

use ::server::server;
use cmd_test::{read_assert_eq, write_cmd};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use tracing::Level;

async fn start_server() -> SocketAddr {
    let _ = tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(Level::INFO)
        .try_init();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { server::run(listener, tokio::signal::ctrl_c()).await });

    addr
}

#[tokio::test]
async fn test_connect() {
    let addr = start_server().await;
    let mut v = vec![];
    // fixme 这里数字大一些 会报错
    for i in 0..100 {
        let h = tokio::spawn(async move {
            // Establish a connection to the server
            let stream = TcpStream::connect(addr).await.unwrap();
            key_value_get_set(stream, i).await
        });
        v.push(h);
    }
    for ele in v {
        let _ = ele.await;
    }
}

async fn key_value_get_set(mut stream: TcpStream, suffix: usize) {
    let hello = format!("hello{}", suffix);
    let world = format!("world{}", suffix);
    write_cmd(&mut stream, vec!["GET", &hello]).await;
    read_assert_eq(&mut stream, b"$-1\r\n").await;

    write_cmd(&mut stream, vec!["SET", &hello, &world]).await;
    read_assert_eq(&mut stream, b"+OK\r\n").await;

    write_cmd(&mut stream, vec!["GET", &hello]).await;
    stream.shutdown().await.unwrap();
    read_assert_eq(&mut stream, format!("+{}\r\n", world).as_bytes()).await;
    let mut response = [0; 10];
    assert_eq!(0, stream.read(&mut response).await.unwrap());
}
