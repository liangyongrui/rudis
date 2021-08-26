//! 测试redis官网的demo

use std::time::Duration;

use cmd_test::{next_frame_eq, start_server, write_cmd};
use server::Frame;
use tokio::time::sleep;

#[tokio::test]
async fn decr() {
    let mut connection = start_server().await;

    write_cmd(&mut connection.stream, vec!["GET", "decr_{tes}t"]).await;
    next_frame_eq(&mut connection, Frame::Null).await;

    write_cmd(&mut connection.stream, vec!["GET", "decr_test"]).await;
    next_frame_eq(&mut connection, Frame::Null).await;

    write_cmd(&mut connection.stream, vec!["DECR", "decr_test"]).await;
    next_frame_eq(&mut connection, Frame::Integer(-1)).await;

    write_cmd(&mut connection.stream, vec!["GET", "decr_test"]).await;
    next_frame_eq(&mut connection, Frame::Simple("-1".into())).await;

    write_cmd(&mut connection.stream, vec!["SET", "mykey", "10"]).await;
    next_frame_eq(&mut connection, Frame::ok()).await;

    write_cmd(&mut connection.stream, vec!["DECR", "mykey"]).await;
    next_frame_eq(&mut connection, Frame::Integer(9)).await;

    write_cmd(
        &mut connection.stream,
        vec!["SET", "mykey", "234293482390480948029348230948"],
    )
    .await;
    next_frame_eq(&mut connection, Frame::ok()).await;

    write_cmd(&mut connection.stream, vec!["DECR", "mykey"]).await;
    next_frame_eq(
        &mut connection,
        Frame::Error("number too large to fit in target type".to_owned()),
    )
    .await;
    dbg!("end1");

    write_cmd(&mut connection.stream, vec!["DECR", "mykey"]).await;
    next_frame_eq(
        &mut connection,
        Frame::Error("number too large to fit in target type".to_owned()),
    )
    .await;
    dbg!("end2");
}

#[tokio::test]
async fn decrby() {
    let mut connection = start_server().await;

    write_cmd(&mut connection.stream, vec!["SET", "mykey", "10"]).await;
    next_frame_eq(&mut connection, Frame::ok()).await;

    write_cmd(&mut connection.stream, vec!["DECRBY", "mykey", "3"]).await;
    next_frame_eq(&mut connection, Frame::Integer(7)).await;
}

#[tokio::test]
async fn del() {
    let mut connection = start_server().await;

    write_cmd(&mut connection.stream, vec!["SET", "key1", "Hello"]).await;
    next_frame_eq(&mut connection, Frame::ok()).await;

    write_cmd(&mut connection.stream, vec!["SET", "key2", "World"]).await;
    next_frame_eq(&mut connection, Frame::ok()).await;

    write_cmd(&mut connection.stream, vec!["DEL", "key1", "key2", "key3"]).await;
    next_frame_eq(&mut connection, Frame::Integer(2)).await;
}

#[tokio::test]
async fn exists() {
    let mut connection = start_server().await;

    write_cmd(&mut connection.stream, vec!["SET", "key1", "Hello"]).await;
    next_frame_eq(&mut connection, Frame::ok()).await;

    write_cmd(&mut connection.stream, vec!["EXISTS", "key1"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["EXISTS", "nosuchkey"]).await;
    next_frame_eq(&mut connection, Frame::Integer(0)).await;

    write_cmd(&mut connection.stream, vec!["SET", "key2", "World"]).await;
    next_frame_eq(&mut connection, Frame::ok()).await;

    write_cmd(
        &mut connection.stream,
        vec!["EXISTS", "key1", "key2", "nosuchkey"],
    )
    .await;
    next_frame_eq(&mut connection, Frame::Integer(2)).await;
}

#[tokio::test]
async fn expire() {
    let mut connection = start_server().await;

    write_cmd(&mut connection.stream, vec!["SET", "mykey", "Hello"]).await;
    next_frame_eq(&mut connection, Frame::ok()).await;

    write_cmd(&mut connection.stream, vec!["EXPIRE", "mykey", "10"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["TTL", "mykey"]).await;
    next_frame_eq(&mut connection, Frame::Integer(10)).await;

    write_cmd(&mut connection.stream, vec!["SET", "mykey", "Hello World"]).await;
    next_frame_eq(&mut connection, Frame::ok()).await;

    write_cmd(&mut connection.stream, vec!["TTL", "mykey"]).await;
    next_frame_eq(&mut connection, Frame::Integer(-1)).await;

    write_cmd(&mut connection.stream, vec!["EXPIRE", "mykey", "10", "XX"]).await;
    next_frame_eq(&mut connection, Frame::Integer(0)).await;

    write_cmd(&mut connection.stream, vec!["TTL", "mykey"]).await;
    next_frame_eq(&mut connection, Frame::Integer(-1)).await;

    write_cmd(&mut connection.stream, vec!["EXPIRE", "mykey", "10", "NX"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["TTL", "mykey"]).await;
    next_frame_eq(&mut connection, Frame::Integer(10)).await;
}

#[tokio::test]
async fn expireat() {
    let mut connection = start_server().await;

    write_cmd(&mut connection.stream, vec!["SET", "mykey", "Hello"]).await;
    next_frame_eq(&mut connection, Frame::ok()).await;

    write_cmd(&mut connection.stream, vec!["EXISTS", "mykey"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(
        &mut connection.stream,
        vec!["EXPIREAT", "mykey", "1293840000"],
    )
    .await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["EXISTS", "mykey"]).await;
    next_frame_eq(&mut connection, Frame::Integer(0)).await;
}

#[tokio::test]
async fn get() {
    let mut connection = start_server().await;

    write_cmd(&mut connection.stream, vec!["GET", "nonexisting"]).await;
    next_frame_eq(&mut connection, Frame::Null).await;

    write_cmd(&mut connection.stream, vec!["SET", "mykey", "Hello"]).await;
    next_frame_eq(&mut connection, Frame::ok()).await;

    write_cmd(&mut connection.stream, vec!["GET", "mykey"]).await;
    next_frame_eq(&mut connection, Frame::Bulk(b"Hello"[..].into())).await;
}

#[tokio::test]
async fn incr() {
    let mut connection = start_server().await;

    write_cmd(&mut connection.stream, vec!["SET", "mykey", "10"]).await;
    next_frame_eq(&mut connection, Frame::ok()).await;

    write_cmd(&mut connection.stream, vec!["INCR", "mykey"]).await;
    next_frame_eq(&mut connection, Frame::Integer(11)).await;

    write_cmd(&mut connection.stream, vec!["GET", "mykey"]).await;
    next_frame_eq(&mut connection, Frame::Simple("11".into())).await;
}

#[tokio::test]
async fn incr_by() {
    let mut connection = start_server().await;

    write_cmd(&mut connection.stream, vec!["SET", "mykey", "10"]).await;
    next_frame_eq(&mut connection, Frame::ok()).await;

    write_cmd(&mut connection.stream, vec!["INCRBY", "mykey", "5"]).await;
    next_frame_eq(&mut connection, Frame::Integer(15)).await;
}
#[tokio::test]
async fn pexpire() {
    let mut connection = start_server().await;

    write_cmd(&mut connection.stream, vec!["SET", "mykey", "Hello"]).await;
    next_frame_eq(&mut connection, Frame::ok()).await;

    write_cmd(&mut connection.stream, vec!["PEXPIRE", "mykey", "1500"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    sleep(Duration::from_millis(1)).await;
    write_cmd(&mut connection.stream, vec!["TTL", "mykey"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    // Not good to test
    // write_cmd(&mut connection.stream, vec!["PTTL", "mykey"]).await;
    // next_frame_eq(&mut connection, b":1496\r\n").await;

    write_cmd(
        &mut connection.stream,
        vec!["PEXPIRE", "mykey", "1000", "XX"],
    )
    .await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["TTL", "mykey"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(
        &mut connection.stream,
        vec!["EXPIRE", "mykey", "1000", "NX"],
    )
    .await;
    next_frame_eq(&mut connection, Frame::Integer(0)).await;

    write_cmd(&mut connection.stream, vec!["TTL", "mykey"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;
}

#[tokio::test]
async fn pexpireat() {
    let mut connection = start_server().await;

    write_cmd(&mut connection.stream, vec!["SET", "mykey", "Hello"]).await;
    next_frame_eq(&mut connection, Frame::ok()).await;

    write_cmd(
        &mut connection.stream,
        vec!["PEXPIREAT", "mykey", "1555555555005"],
    )
    .await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["TTL", "mykey"]).await;
    next_frame_eq(&mut connection, Frame::Integer(-2)).await;

    write_cmd(&mut connection.stream, vec!["PTTL", "mykey"]).await;
    next_frame_eq(&mut connection, Frame::Integer(-2)).await;
}

#[tokio::test]
async fn psetex() {
    let mut connection = start_server().await;

    write_cmd(
        &mut connection.stream,
        vec!["PSETEX", "mykey", "90", "Hello"],
    )
    .await;
    next_frame_eq(&mut connection, Frame::ok()).await;

    // Not good to test
    // write_cmd(&mut connection.stream, vec!["PTTL", "mykey"]).await;
    // next_frame_eq(&mut connection, Frame::Integer(90)).await;

    write_cmd(&mut connection.stream, vec!["GET", "mykey"]).await;
    next_frame_eq(&mut connection, Frame::Bulk(b"Hello"[..].into())).await;

    sleep(Duration::from_millis(100)).await;
    write_cmd(&mut connection.stream, vec!["GET", "mykey"]).await;
    next_frame_eq(&mut connection, Frame::Null).await;
}

#[tokio::test]
async fn set() {
    let mut connection = start_server().await;

    write_cmd(&mut connection.stream, vec!["SET", "mykey", "Hello"]).await;
    next_frame_eq(&mut connection, Frame::ok()).await;

    write_cmd(&mut connection.stream, vec!["GET", "mykey"]).await;
    next_frame_eq(&mut connection, Frame::Bulk(b"Hello"[..].into())).await;

    write_cmd(
        &mut connection.stream,
        vec!["SET", "anotherkey", "will expire in a minute", "EX", "60"],
    )
    .await;
    next_frame_eq(&mut connection, Frame::ok()).await;
}

#[tokio::test]
async fn setex() {
    let mut connection = start_server().await;

    write_cmd(
        &mut connection.stream,
        vec!["SETEX", "mykey", "90", "Hello"],
    )
    .await;
    next_frame_eq(&mut connection, Frame::ok()).await;

    write_cmd(&mut connection.stream, vec!["TTL", "mykey"]).await;
    next_frame_eq(&mut connection, Frame::Integer(90)).await;

    write_cmd(&mut connection.stream, vec!["GET", "mykey"]).await;
    next_frame_eq(&mut connection, Frame::Bulk(b"Hello"[..].into())).await;
}
