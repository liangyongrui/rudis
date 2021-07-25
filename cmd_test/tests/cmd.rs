//! 每个命令简单测一下功能是否正常，无需关联
//! 命令功能的详细测试，在db模块测

use std::sync::Arc;

use cmd_test::start_server;
use component::Frame;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::test]
async fn decr() {
    let mut stream = start_server().await;
    let key: Arc<str> = "decr_test".to_owned().into();
    let get: Vec<u8> = Frame::Array(vec![
        Frame::Simple("GET".to_owned()),
        Frame::Simple(key.to_string()),
    ])
    .into();
    stream.write_all(&get).await.unwrap();
    // Read nil response
    let mut response = [0; 5];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(b"$-1\r\n", &response);
    let decr: Vec<u8> = Frame::Array(vec![
        Frame::Simple("DECR".to_owned()),
        Frame::Simple(key.to_string()),
    ])
    .into();
    stream.write_all(&decr).await.unwrap();
    let ans = format!(":{}\r\n", "-1");
    let mut response = vec![0; ans.len()];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(ans.as_bytes(), &response);

    stream.write_all(&get).await.unwrap();
    // Shutdown the write half
    stream.shutdown().await.unwrap();

    let ans = format!(":{}\r\n", "-1");
    let mut response = vec![0; ans.len()];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(ans.as_bytes(), &response);

    // Receive `None`
    assert_eq!(0, stream.read(&mut response).await.unwrap());
}
