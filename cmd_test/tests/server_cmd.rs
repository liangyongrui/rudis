//! 测试redis官网的demo

use cmd_test::{next_frame_eq, start_server, write_cmd, write_cmd_bytes};
use common::connection::parse::frame::Frame;

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

#[tokio::test]
async fn dump() {
    let mut connection = start_server().await;

    write_cmd(&mut connection.stream, vec!["SET", "key1", "Hello"]).await;
    next_frame_eq(&mut connection, Frame::ok()).await;

    write_cmd(&mut connection.stream, vec!["dump", "key1"]).await;
    write_cmd(&mut connection.stream, vec!["dump", "key2"]).await;
}

#[tokio::test]
async fn restore() {
    let mut connection = start_server().await;

    write_cmd(&mut connection.stream, vec!["del", "mykey"]).await;
    next_frame_eq(&mut connection, Frame::Integer(0)).await;

    write_cmd(
        &mut connection.stream,
        vec!["rpush", "mykey", "1", "2", "3"],
    )
    .await;
    next_frame_eq(&mut connection, Frame::Integer(3)).await;

    // write_cmd(&mut connection.stream, vec!["dump", "mykey"]).await;

    write_cmd(&mut connection.stream, vec!["del", "mykey"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd_bytes(
        &mut connection.stream,
        vec![
            &b"restore"[..],
            &b"mykey"[..],
            &b"0"[..],
            &[
                6, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 49, 2, 0,
                0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 50, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 51, 0, 0, 0,
                0, 0, 0, 0, 0,
            ],
        ],
    )
    .await;
    next_frame_eq(&mut connection, Frame::ok()).await;

    write_cmd(&mut connection.stream, vec!["LRANGE", "mykey", "0", "-1"]).await;
    next_frame_eq(
        &mut connection,
        Frame::Array(vec![
            Frame::Bulk(b"1"[..].into()),
            Frame::Bulk(b"2"[..].into()),
            Frame::Bulk(b"3"[..].into()),
        ]),
    )
    .await;
}
