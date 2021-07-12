use std::net::SocketAddr;

use rcc::{cmd::Get, server, SimpleType};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use tracing::Level;

async fn start_server() -> SocketAddr {
    tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(Level::DEBUG)
        .try_init()
        .unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move { server::run(listener, tokio::signal::ctrl_c()).await });

    addr
}

#[tokio::test]
async fn key_value_get_set() {
    let addr = start_server().await;
    dbg!(addr);
    // Establish a connection to the server
    let mut stream = TcpStream::connect(addr).await.unwrap();
    // Get a key, data is missing
    let get = Get {
        key: SimpleType::SimpleString("hello".to_string()),
    };
    let cmd = &get.into_cmd()[..];
    assert_eq!(b"*2\r\n$3\r\nGET\r\n$5\r\nhello\r\n", cmd);
    stream.write_all(cmd).await.unwrap();

    // Read nil response
    let mut response = [0; 5];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(b"$-1\r\n", &response);

    // Set a key
    stream
        .write_all(b"*3\r\n$3\r\nSET\r\n$5\r\nhello\r\n$5\r\nworld\r\n")
        .await
        .unwrap();

    // Read OK
    let mut response = [0; 5];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(b"+OK\r\n", &response);

    // Get the key, data is present
    let get = Get {
        key: SimpleType::SimpleString("hello".to_string()),
    };
    stream.write_all(&get.into_cmd()[..]).await.unwrap();

    // Shutdown the write half
    stream.shutdown().await.unwrap();

    // Read "world" response
    let mut response = [0; 11];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(b"$5\r\nworld\r\n", &response);

    // Receive `None`
    assert_eq!(0, stream.read(&mut response).await.unwrap());
}
