//! 测试redis官网的demo

use cmd_test::{next_array_frame_sorted_eq, next_frame_eq, start_server, write_cmd};
use connection::parse::frame::Frame;

#[tokio::test]
async fn sadd() {
    let mut connection = start_server().await;

    write_cmd(&mut connection.stream, vec!["SADD", "myset", "Hello"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["SADD", "myset", "World"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["SADD", "myset", "World"]).await;
    next_frame_eq(&mut connection, Frame::Integer(0)).await;

    write_cmd(&mut connection.stream, vec!["SMEMBERS", "myset"]).await;
    next_array_frame_sorted_eq(
        &mut connection,
        vec![
            Frame::Simple(b"Hello"[..].into()),
            Frame::Simple(b"World"[..].into()),
        ],
    )
    .await;
}

#[tokio::test]
async fn sismember() {
    let mut connection = start_server().await;

    write_cmd(&mut connection.stream, vec!["SADD", "myset", "one"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["SISMEMBER", "myset", "one"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["SISMEMBER", "myset", "two"]).await;
    next_frame_eq(&mut connection, Frame::Integer(0)).await;
}

#[tokio::test]
async fn smembers() {
    let mut connection = start_server().await;

    write_cmd(&mut connection.stream, vec!["SADD", "myset", "Hello"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["SADD", "myset", "World"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["SMEMBERS", "myset"]).await;
    next_array_frame_sorted_eq(
        &mut connection,
        vec![
            Frame::Simple(b"Hello"[..].into()),
            Frame::Simple(b"World"[..].into()),
        ],
    )
    .await;
}

#[tokio::test]
async fn smismember() {
    let mut connection = start_server().await;

    write_cmd(&mut connection.stream, vec!["SADD", "myset", "one"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["SADD", "myset", "one"]).await;
    next_frame_eq(&mut connection, Frame::Integer(0)).await;

    write_cmd(
        &mut connection.stream,
        vec!["SMISMEMBER", "myset", "one", "notamember"],
    )
    .await;
    next_array_frame_sorted_eq(&mut connection, vec![Frame::Integer(1), Frame::Integer(0)]).await;
}

#[tokio::test]
async fn srem() {
    let mut connection = start_server().await;

    write_cmd(&mut connection.stream, vec!["SADD", "myset", "one"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["SADD", "myset", "two"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["SADD", "myset", "three"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["SREM", "myset", "one"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["SREM", "myset", "four"]).await;
    next_frame_eq(&mut connection, Frame::Integer(0)).await;

    write_cmd(&mut connection.stream, vec!["SMEMBERS", "myset"]).await;
    next_array_frame_sorted_eq(
        &mut connection,
        vec![
            Frame::Simple(b"two"[..].into()),
            Frame::Simple(b"three"[..].into()),
        ],
    )
    .await;
}
