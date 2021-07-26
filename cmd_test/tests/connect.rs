//! 主要测试 连接过程

use std::net::SocketAddr;

use component::{server, Frame};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use tracing::{debug, Level};

async fn start_server() -> SocketAddr {
    let _ = tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(Level::DEBUG)
        .try_init();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    debug!(?addr);
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
    // Get a key, data is missing
    let get: Vec<u8> = Frame::Array(vec![
        Frame::Simple("GET".to_owned()),
        Frame::Bulk(hello.clone().into()),
    ])
    .into();
    stream.write_all(&get).await.unwrap();

    // Read nil response
    let mut response = [0; 5];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(b"$-1\r\n", &response);
    let set: Vec<u8> = Frame::Array(vec![
        Frame::Bulk("SET".into()),
        Frame::Bulk(hello.into()),
        Frame::Bulk(world.clone().into()),
    ])
    .into();
    // Set a key
    stream.write_all(&set).await.unwrap();

    // Read OK
    let mut response = [0; 5];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(b"+OK\r\n", &response);

    stream.write_all(&get).await.unwrap();

    // Shutdown the write half
    stream.shutdown().await.unwrap();

    // Read "world" response
    let mut response = vec![0; format!("${}\r\n{}\r\n", world.len(), world).len()];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(
        format!("${}\r\n{}\r\n", world.len(), world).as_bytes(),
        &response
    );

    // Receive `None`
    assert_eq!(0, stream.read(&mut response).await.unwrap());
}
