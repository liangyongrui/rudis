//! 测试redis官网的demo

use cmd_test::{next_frame_eq, start_server, write_cmd};
use connection::parse::frame::Frame;

#[tokio::test]
async fn llen() {
    let mut connection = start_server().await;

    write_cmd(&mut connection.stream, vec!["LPUSH", "mylist", "World"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["LPUSH", "mylist", "Hello"]).await;
    next_frame_eq(&mut connection, Frame::Integer(2)).await;

    write_cmd(&mut connection.stream, vec!["LLEN", "mylist"]).await;
    next_frame_eq(&mut connection, Frame::Integer(2)).await;
}

#[tokio::test]
async fn lpop() {
    let mut connection = start_server().await;

    write_cmd(
        &mut connection.stream,
        vec!["RPUSH", "mylist", "one", "two", "three", "four", "five"],
    )
    .await;
    next_frame_eq(&mut connection, Frame::Integer(5)).await;

    write_cmd(&mut connection.stream, vec!["LPOP", "mylist"]).await;
    next_frame_eq(
        &mut connection,
        Frame::Array(vec![Frame::Bulk(b"one"[..].into())]),
    )
    .await;

    write_cmd(&mut connection.stream, vec!["LPOP", "mylist", "2"]).await;
    next_frame_eq(
        &mut connection,
        Frame::Array(vec![
            Frame::Bulk(b"two"[..].into()),
            Frame::Bulk(b"three"[..].into()),
        ]),
    )
    .await;

    write_cmd(&mut connection.stream, vec!["LRANGE", "mylist", "0", "-1"]).await;
    next_frame_eq(
        &mut connection,
        Frame::Array(vec![
            Frame::Bulk(b"four"[..].into()),
            Frame::Bulk(b"five"[..].into()),
        ]),
    )
    .await;
}

#[tokio::test]
async fn lpush() {
    let mut connection = start_server().await;

    write_cmd(&mut connection.stream, vec!["LPUSH", "mylist", "world"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;
    write_cmd(&mut connection.stream, vec!["LPUSH", "mylist", "hello"]).await;
    next_frame_eq(&mut connection, Frame::Integer(2)).await;

    write_cmd(&mut connection.stream, vec!["LRANGE", "mylist", "0", "-1"]).await;
    next_frame_eq(
        &mut connection,
        Frame::Array(vec![
            Frame::Bulk(b"hello"[..].into()),
            Frame::Bulk(b"world"[..].into()),
        ]),
    )
    .await;
}

#[tokio::test]
async fn lpushx() {
    let mut connection = start_server().await;

    write_cmd(&mut connection.stream, vec!["LPUSH", "mylist", "World"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["LPUSHX", "mylist", "Hello"]).await;
    next_frame_eq(&mut connection, Frame::Integer(2)).await;

    write_cmd(
        &mut connection.stream,
        vec!["LPUSHX", "myotherlist", "Hello"],
    )
    .await;
    next_frame_eq(&mut connection, Frame::Integer(0)).await;

    write_cmd(&mut connection.stream, vec!["LRANGE", "mylist", "0", "-1"]).await;
    next_frame_eq(
        &mut connection,
        Frame::Array(vec![
            Frame::Bulk(b"Hello"[..].into()),
            Frame::Bulk(b"World"[..].into()),
        ]),
    )
    .await;

    write_cmd(
        &mut connection.stream,
        vec!["LRANGE", "myotherlist", "0", "-1"],
    )
    .await;
    next_frame_eq(&mut connection, Frame::Array(vec![])).await;
}

#[tokio::test]
async fn lrange() {
    let mut connection = start_server().await;

    write_cmd(&mut connection.stream, vec!["RPUSH", "mylist", "one"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["RPUSH", "mylist", "two"]).await;
    next_frame_eq(&mut connection, Frame::Integer(2)).await;

    write_cmd(&mut connection.stream, vec!["RPUSH", "mylist", "three"]).await;
    next_frame_eq(&mut connection, Frame::Integer(3)).await;

    write_cmd(&mut connection.stream, vec!["LRANGE", "mylist", "0", "0"]).await;
    next_frame_eq(
        &mut connection,
        Frame::Array(vec![Frame::Bulk(b"one"[..].into())]),
    )
    .await;

    write_cmd(&mut connection.stream, vec!["LRANGE", "mylist", "-3", "2"]).await;
    next_frame_eq(
        &mut connection,
        Frame::Array(vec![
            Frame::Bulk(b"one"[..].into()),
            Frame::Bulk(b"two"[..].into()),
            Frame::Bulk(b"three"[..].into()),
        ]),
    )
    .await;

    write_cmd(
        &mut connection.stream,
        vec!["LRANGE", "mylist", "-100", "100"],
    )
    .await;
    next_frame_eq(
        &mut connection,
        Frame::Array(vec![
            Frame::Bulk(b"one"[..].into()),
            Frame::Bulk(b"two"[..].into()),
            Frame::Bulk(b"three"[..].into()),
        ]),
    )
    .await;

    write_cmd(&mut connection.stream, vec!["LRANGE", "mylist", "5", "10"]).await;
    next_frame_eq(&mut connection, Frame::Array(vec![])).await;
}

#[tokio::test]
async fn rpop() {
    let mut connection = start_server().await;

    write_cmd(
        &mut connection.stream,
        vec!["RPUSH", "mylist", "one", "two", "three", "four", "five"],
    )
    .await;
    next_frame_eq(&mut connection, Frame::Integer(5)).await;

    write_cmd(&mut connection.stream, vec!["RPOP", "mylist"]).await;
    next_frame_eq(
        &mut connection,
        Frame::Array(vec![Frame::Bulk(b"five"[..].into())]),
    )
    .await;

    write_cmd(&mut connection.stream, vec!["RPOP", "mylist", "2"]).await;
    next_frame_eq(
        &mut connection,
        Frame::Array(vec![
            Frame::Bulk(b"four"[..].into()),
            Frame::Bulk(b"three"[..].into()),
        ]),
    )
    .await;

    write_cmd(&mut connection.stream, vec!["LRANGE", "mylist", "0", "-1"]).await;
    next_frame_eq(
        &mut connection,
        Frame::Array(vec![
            Frame::Bulk(b"one"[..].into()),
            Frame::Bulk(b"two"[..].into()),
        ]),
    )
    .await;
}

#[tokio::test]
async fn rpush() {
    let mut connection = start_server().await;

    write_cmd(&mut connection.stream, vec!["RPUSH", "mylist", "hello"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;
    write_cmd(&mut connection.stream, vec!["RPUSH", "mylist", "world"]).await;
    next_frame_eq(&mut connection, Frame::Integer(2)).await;

    write_cmd(&mut connection.stream, vec!["LRANGE", "mylist", "0", "-1"]).await;
    next_frame_eq(
        &mut connection,
        Frame::Array(vec![
            Frame::Bulk(b"hello"[..].into()),
            Frame::Bulk(b"world"[..].into()),
        ]),
    )
    .await;
}

#[tokio::test]
async fn rpushx() {
    let mut connection = start_server().await;

    write_cmd(&mut connection.stream, vec!["RPUSH", "mylist", "Hello"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["RPUSHX", "mylist", "World"]).await;
    next_frame_eq(&mut connection, Frame::Integer(2)).await;

    write_cmd(
        &mut connection.stream,
        vec!["RPUSHX", "myotherlist", "World"],
    )
    .await;
    next_frame_eq(&mut connection, Frame::Integer(0)).await;

    write_cmd(&mut connection.stream, vec!["LRANGE", "mylist", "0", "-1"]).await;
    next_frame_eq(
        &mut connection,
        Frame::Array(vec![
            Frame::Bulk(b"Hello"[..].into()),
            Frame::Bulk(b"World"[..].into()),
        ]),
    )
    .await;

    write_cmd(
        &mut connection.stream,
        vec!["LRANGE", "myotherlist", "0", "-1"],
    )
    .await;
    next_frame_eq(&mut connection, Frame::Array(vec![])).await;
}
