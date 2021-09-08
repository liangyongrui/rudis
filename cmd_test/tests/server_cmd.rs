//! 测试redis官网的demo

use cmd_test::{next_frame_eq, start_server, write_cmd};
use connection::parse::frame::Frame;

#[tokio::test]
async fn flushall() {
    let mut connection = start_server().await;

    write_cmd(&mut connection.stream, vec!["SET", "key1", "Hello"]).await;
    next_frame_eq(&mut connection, Frame::ok()).await;

    write_cmd(&mut connection.stream, vec!["EXISTS", "key1"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["flushall"]).await;
    next_frame_eq(&mut connection, Frame::ok()).await;

    write_cmd(&mut connection.stream, vec!["EXISTS", "key1"]).await;
    next_frame_eq(&mut connection, Frame::Integer(0)).await;

    write_cmd(&mut connection.stream, vec!["SET", "key2", "Hello"]).await;
    next_frame_eq(&mut connection, Frame::ok()).await;

    write_cmd(&mut connection.stream, vec!["EXISTS", "key2"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["flushall", "sync"]).await;
    next_frame_eq(&mut connection, Frame::ok()).await;

    write_cmd(&mut connection.stream, vec!["EXISTS", "key1", "key2"]).await;
    next_frame_eq(&mut connection, Frame::Integer(0)).await;
}

#[tokio::test]
async fn info() {
    let mut connection = start_server().await;

    write_cmd(&mut connection.stream, vec!["INFO"]).await;
}
