//! 每个命令简单测一下功能是否正常，无需关联
//! 命令功能的详细测试，在db模块测

use std::sync::Arc;

use rcc::{
    cmd::{Decr, Get},
    server, SimpleType,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use tracing::{debug, Level};

async fn start_server() -> TcpStream {
    tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(Level::DEBUG)
        .try_init()
        .unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    debug!("789");

    tokio::spawn(async move { server::run(listener, tokio::signal::ctrl_c()).await });
    debug!("aaaa");
    TcpStream::connect(addr).await.unwrap()
}

#[tokio::test]
async fn decr() {
    let mut stream = start_server().await;
    debug!("123");
    let key: Arc<str> = "decr_test".to_owned().into();
    let get = Get {
        key: SimpleType::String(key.clone()),
    };
    let cmd = &get.into_cmd_bytes()[..];
    stream.write_all(cmd).await.unwrap();
    debug!("456");
    // Read nil response
    let mut response = [0; 5];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(b"$-1\r\n", &response);
    let decr = Decr {
        key: SimpleType::String(key.clone()),
    };
    stream.write_all(&decr.into_cmd_bytes()[..]).await.unwrap();
    let ans = format!(":{}\r\n", "-1");
    let mut response = vec![0; ans.len()];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(ans.as_bytes(), &response);

    stream.write_all(cmd).await.unwrap();
    // Shutdown the write half
    stream.shutdown().await.unwrap();

    let ans = format!(":{}\r\n", "-1");
    let mut response = vec![0; ans.len()];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(ans.as_bytes(), &response);

    // Receive `None`
    assert_eq!(0, stream.read(&mut response).await.unwrap());
}
