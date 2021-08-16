//! 测试redis官网的demo

use cmd_test::{next_array_frame_sorted_eq, next_frame_eq, start_server, write_cmd};
use server::Frame;

#[tokio::test]
async fn hdel() {
    let mut connection = start_server().await;

    write_cmd(
        &mut connection.stream,
        vec!["HSET", "myhash", "field1", "foo"],
    )
    .await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["HDEL", "myhash", "field1"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["HDEL", "myhash", "field2"]).await;
    next_frame_eq(&mut connection, Frame::Integer(0)).await;
}

#[tokio::test]
async fn hexists() {
    let mut connection = start_server().await;

    write_cmd(
        &mut connection.stream,
        vec!["HSET", "myhash", "field1", "foo"],
    )
    .await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["HEXISTS", "myhash", "field1"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["HEXISTS", "myhash", "field2"]).await;
    next_frame_eq(&mut connection, Frame::Integer(0)).await;
}

#[tokio::test]
async fn hget() {
    let mut connection = start_server().await;

    write_cmd(
        &mut connection.stream,
        vec!["HSET", "myhash", "field1", "foo"],
    )
    .await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["HGET", "myhash", "field1"]).await;
    next_frame_eq(&mut connection, Frame::Bulk(b"foo"[..].into())).await;

    write_cmd(&mut connection.stream, vec!["HGET", "myhash", "field2"]).await;
    next_frame_eq(&mut connection, Frame::Null).await;
}

#[tokio::test]
async fn hgetll() {
    let mut connection = start_server().await;

    write_cmd(
        &mut connection.stream,
        vec!["HSET", "myhash", "field1", "Hello"],
    )
    .await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(
        &mut connection.stream,
        vec!["HSET", "myhash", "field2", "World"],
    )
    .await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["HGETALL", "myhash"]).await;
    let res = vec![
        Frame::Bulk(b"field1"[..].into()),
        Frame::Bulk(b"Hello"[..].into()),
        Frame::Bulk(b"field2"[..].into()),
        Frame::Bulk(b"World"[..].into()),
    ];
    next_array_frame_sorted_eq(&mut connection, res).await;
}

#[tokio::test]
async fn hincrby() {
    let mut connection = start_server().await;

    write_cmd(&mut connection.stream, vec!["HSET", "myhash", "field", "5"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(
        &mut connection.stream,
        vec!["HINCRBY", "myhash", "field", "1"],
    )
    .await;
    next_frame_eq(&mut connection, Frame::Integer(6)).await;

    write_cmd(
        &mut connection.stream,
        vec!["HINCRBY", "myhash", "field", "-1"],
    )
    .await;
    next_frame_eq(&mut connection, Frame::Integer(5)).await;

    write_cmd(
        &mut connection.stream,
        vec!["HINCRBY", "myhash", "field", "-10"],
    )
    .await;
    next_frame_eq(&mut connection, Frame::Integer(-5)).await;
}

#[tokio::test]
async fn hmget() {
    let mut connection = start_server().await;

    write_cmd(
        &mut connection.stream,
        vec!["HSET", "myhash", "field1", "Hello"],
    )
    .await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(
        &mut connection.stream,
        vec!["HSET", "myhash", "field2", "World"],
    )
    .await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(
        &mut connection.stream,
        vec!["HMGET", "myhash", "field1", "field2", "field3"],
    )
    .await;
    let res = Frame::Array(vec![
        Frame::Bulk(b"Hello"[..].into()),
        Frame::Bulk(b"World"[..].into()),
        Frame::Null,
    ]);
    next_frame_eq(&mut connection, res).await;
}

#[tokio::test]
async fn hset() {
    let mut connection = start_server().await;

    write_cmd(
        &mut connection.stream,
        vec!["HSET", "myhash", "field1", "Hello"],
    )
    .await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["HGET", "myhash", "field1"]).await;
    next_frame_eq(&mut connection, Frame::Bulk(b"Hello"[..].into())).await;
}

#[tokio::test]
async fn hsetnx() {
    let mut connection = start_server().await;

    write_cmd(
        &mut connection.stream,
        vec!["hsetnx", "myhash", "field", "Hello"],
    )
    .await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(
        &mut connection.stream,
        vec!["hsetnx", "myhash", "field", "World"],
    )
    .await;
    next_frame_eq(&mut connection, Frame::Integer(0)).await;

    write_cmd(&mut connection.stream, vec!["HGET", "myhash", "field"]).await;
    next_frame_eq(&mut connection, Frame::Bulk(b"Hello"[..].into())).await;
}
