//! 每个命令简单测一下功能是否正常，无需关联
//! 命令功能的详细测试，在db模块测

use rcc::{
    cmd::{Decr, Get},
    server, SimpleType,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use tracing::Level;

async fn start_server() -> TcpStream {
    tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(Level::DEBUG)
        .try_init()
        .unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { server::run(listener, tokio::signal::ctrl_c()).await });
    TcpStream::connect(addr).await.unwrap()
}

#[tokio::test]
async fn decr() {
    let mut stream = start_server().await;
    let key = "decr_test".to_owned();
    let get = Get {
        key: SimpleType::SimpleString(key.clone()),
    };
    let cmd = &get.into_cmd()[..];
    stream.write_all(cmd).await.unwrap();
    // Read nil response
    let mut response = [0; 5];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(b"$-1\r\n", &response);
    let decr = Decr {
        key: SimpleType::SimpleString(key),
    };
    stream.write_all(&decr.into_cmd()[..]).await.unwrap();
    // Read OK
    let mut response = [0; 5];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(b"+OK\r\n", &response);
    stream.write_all(cmd).await.unwrap();
    // Shutdown the write half
    stream.shutdown().await.unwrap();

    let ans = format!("${}\r\n{}\r\n", "-1".len(), "-1");
    let mut response = vec![0; ans.len()];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(ans.as_bytes(), &response);

    // Receive `None`
    assert_eq!(0, stream.read(&mut response).await.unwrap());
}
