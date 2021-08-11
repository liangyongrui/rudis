//! 测试redis官网的demo

use std::time::Duration;

use cmd_test::{read_assert_eq, start_server, write_cmd};
use tokio::time::sleep;

#[tokio::test]
async fn decr() {
    let mut stream = start_server().await;

    write_cmd(&mut stream, vec!["GET", "decr_test"]).await;
    read_assert_eq(&mut stream, b"$-1\r\n").await;

    write_cmd(&mut stream, vec!["DECR", "decr_test"]).await;
    read_assert_eq(&mut stream, b":-1\r\n").await;

    write_cmd(&mut stream, vec!["GET", "decr_test"]).await;
    read_assert_eq(&mut stream, b"+-1\r\n").await;

    write_cmd(&mut stream, vec!["SET", "mykey", "10"]).await;
    read_assert_eq(&mut stream, b"+OK\r\n").await;

    write_cmd(&mut stream, vec!["DECR", "mykey"]).await;
    read_assert_eq(&mut stream, b":9\r\n").await;

    write_cmd(
        &mut stream,
        vec!["SET", "mykey", "234293482390480948029348230948"],
    )
    .await;
    read_assert_eq(&mut stream, b"+OK\r\n").await;

    write_cmd(&mut stream, vec!["DECR", "mykey"]).await;
    read_assert_eq(&mut stream, b"-number too large to fit in target type\r\n").await;
    dbg!("end1");

    write_cmd(&mut stream, vec!["DECR", "mykey"]).await;
    read_assert_eq(&mut stream, b"-number too large to fit in target type\r\n").await;
    dbg!("end2");
}

#[tokio::test]
async fn decrby() {
    let mut stream = start_server().await;

    write_cmd(&mut stream, vec!["SET", "mykey", "10"]).await;
    read_assert_eq(&mut stream, b"+OK\r\n").await;

    write_cmd(&mut stream, vec!["DECRBY", "mykey", "3"]).await;
    read_assert_eq(&mut stream, b":7\r\n").await;
}

#[tokio::test]
async fn del() {
    let mut stream = start_server().await;

    write_cmd(&mut stream, vec!["SET", "key1", "Hello"]).await;
    read_assert_eq(&mut stream, b"+OK\r\n").await;

    write_cmd(&mut stream, vec!["SET", "key2", "World"]).await;
    read_assert_eq(&mut stream, b"+OK\r\n").await;

    write_cmd(&mut stream, vec!["DEL", "key1", "key2", "key3"]).await;
    read_assert_eq(&mut stream, b":2\r\n").await;
}

#[tokio::test]
async fn exists() {
    let mut stream = start_server().await;

    write_cmd(&mut stream, vec!["SET", "key1", "Hello"]).await;
    read_assert_eq(&mut stream, b"+OK\r\n").await;

    write_cmd(&mut stream, vec!["EXISTS", "key1"]).await;
    read_assert_eq(&mut stream, b":1\r\n").await;

    write_cmd(&mut stream, vec!["EXISTS", "nosuchkey"]).await;
    read_assert_eq(&mut stream, b":0\r\n").await;

    write_cmd(&mut stream, vec!["SET", "key2", "World"]).await;
    read_assert_eq(&mut stream, b"+OK\r\n").await;

    write_cmd(&mut stream, vec!["EXISTS", "key1", "key2", "nosuchkey"]).await;
    read_assert_eq(&mut stream, b":2\r\n").await;
}

#[tokio::test]
async fn expire() {
    let mut stream = start_server().await;

    write_cmd(&mut stream, vec!["SET", "mykey", "Hello"]).await;
    read_assert_eq(&mut stream, b"+OK\r\n").await;

    write_cmd(&mut stream, vec!["EXPIRE", "mykey", "10"]).await;
    read_assert_eq(&mut stream, b":1\r\n").await;

    write_cmd(&mut stream, vec!["TTL", "mykey"]).await;
    read_assert_eq(&mut stream, b":10\r\n").await;

    write_cmd(&mut stream, vec!["SET", "mykey", "Hello World"]).await;
    read_assert_eq(&mut stream, b"+OK\r\n").await;

    write_cmd(&mut stream, vec!["TTL", "mykey"]).await;
    read_assert_eq(&mut stream, b":-1\r\n").await;

    write_cmd(&mut stream, vec!["EXPIRE", "mykey", "10", "XX"]).await;
    read_assert_eq(&mut stream, b":0\r\n").await;

    write_cmd(&mut stream, vec!["TTL", "mykey"]).await;
    read_assert_eq(&mut stream, b":-1\r\n").await;

    write_cmd(&mut stream, vec!["EXPIRE", "mykey", "10", "NX"]).await;
    read_assert_eq(&mut stream, b":1\r\n").await;

    write_cmd(&mut stream, vec!["TTL", "mykey"]).await;
    read_assert_eq(&mut stream, b":10\r\n").await;
}

#[tokio::test]
async fn expireat() {
    let mut stream = start_server().await;

    write_cmd(&mut stream, vec!["SET", "mykey", "Hello"]).await;
    read_assert_eq(&mut stream, b"+OK\r\n").await;

    write_cmd(&mut stream, vec!["EXISTS", "mykey"]).await;
    read_assert_eq(&mut stream, b":1\r\n").await;

    write_cmd(&mut stream, vec!["EXPIREAT", "mykey", "1293840000"]).await;
    read_assert_eq(&mut stream, b":1\r\n").await;

    write_cmd(&mut stream, vec!["EXISTS", "mykey"]).await;
    read_assert_eq(&mut stream, b":0\r\n").await;
}

#[tokio::test]
async fn get() {
    let mut stream = start_server().await;

    write_cmd(&mut stream, vec!["GET", "nonexisting"]).await;
    read_assert_eq(&mut stream, b"$-1\r\n").await;

    write_cmd(&mut stream, vec!["SET", "mykey", "Hello"]).await;
    read_assert_eq(&mut stream, b"+OK\r\n").await;

    write_cmd(&mut stream, vec!["GET", "mykey"]).await;
    read_assert_eq(&mut stream, b"+Hello\r\n").await;
}

#[tokio::test]
async fn incr() {
    let mut stream = start_server().await;

    write_cmd(&mut stream, vec!["SET", "mykey", "10"]).await;
    read_assert_eq(&mut stream, b"+OK\r\n").await;

    write_cmd(&mut stream, vec!["INCR", "mykey"]).await;
    read_assert_eq(&mut stream, b":11\r\n").await;

    write_cmd(&mut stream, vec!["GET", "mykey"]).await;
    read_assert_eq(&mut stream, b"+11\r\n").await;
}

#[tokio::test]
async fn incr_by() {
    let mut stream = start_server().await;

    write_cmd(&mut stream, vec!["SET", "mykey", "10"]).await;
    read_assert_eq(&mut stream, b"+OK\r\n").await;

    write_cmd(&mut stream, vec!["INCRBY", "mykey", "5"]).await;
    read_assert_eq(&mut stream, b":15\r\n").await;
}
#[tokio::test]
async fn pexpire() {
    let mut stream = start_server().await;

    write_cmd(&mut stream, vec!["SET", "mykey", "Hello"]).await;
    read_assert_eq(&mut stream, b"+OK\r\n").await;

    write_cmd(&mut stream, vec!["PEXPIRE", "mykey", "1500"]).await;
    read_assert_eq(&mut stream, b":1\r\n").await;

    sleep(Duration::from_millis(1)).await;
    write_cmd(&mut stream, vec!["TTL", "mykey"]).await;
    read_assert_eq(&mut stream, b":1\r\n").await;

    // Not good to test
    // write_cmd(&mut stream, vec!["PTTL", "mykey"]).await;
    // read_assert_eq(&mut stream, b":1496\r\n").await;

    write_cmd(&mut stream, vec!["PEXPIRE", "mykey", "1000", "XX"]).await;
    read_assert_eq(&mut stream, b":1\r\n").await;

    write_cmd(&mut stream, vec!["TTL", "mykey"]).await;
    read_assert_eq(&mut stream, b":1\r\n").await;

    write_cmd(&mut stream, vec!["EXPIRE", "mykey", "1000", "NX"]).await;
    read_assert_eq(&mut stream, b":0\r\n").await;

    write_cmd(&mut stream, vec!["TTL", "mykey"]).await;
    read_assert_eq(&mut stream, b":1\r\n").await;
}

#[tokio::test]
async fn pexpireat() {
    let mut stream = start_server().await;

    write_cmd(&mut stream, vec!["SET", "mykey", "Hello"]).await;
    read_assert_eq(&mut stream, b"+OK\r\n").await;

    write_cmd(&mut stream, vec!["PEXPIREAT", "mykey", "1555555555005"]).await;
    read_assert_eq(&mut stream, b":1\r\n").await;

    write_cmd(&mut stream, vec!["TTL", "mykey"]).await;
    read_assert_eq(&mut stream, b":-2\r\n").await;

    write_cmd(&mut stream, vec!["PTTL", "mykey"]).await;
    read_assert_eq(&mut stream, b":-2\r\n").await;
}

#[tokio::test]
async fn psetex() {
    let mut stream = start_server().await;

    write_cmd(&mut stream, vec!["PSETEX", "mykey", "90", "Hello"]).await;
    read_assert_eq(&mut stream, b"+OK\r\n").await;

    // Not good to test
    // write_cmd(&mut stream, vec!["PTTL", "mykey"]).await;
    // read_assert_eq(&mut stream, b":90\r\n").await;

    write_cmd(&mut stream, vec!["GET", "mykey"]).await;
    read_assert_eq(&mut stream, b"+Hello\r\n").await;

    sleep(Duration::from_millis(100)).await;
    write_cmd(&mut stream, vec!["GET", "mykey"]).await;
    read_assert_eq(&mut stream, b"$-1\r\n").await;
}

#[tokio::test]
async fn set() {
    let mut stream = start_server().await;

    write_cmd(&mut stream, vec!["SET", "mykey", "Hello"]).await;
    read_assert_eq(&mut stream, b"+OK\r\n").await;

    write_cmd(&mut stream, vec!["GET", "mykey"]).await;
    read_assert_eq(&mut stream, b"+Hello\r\n").await;

    write_cmd(
        &mut stream,
        vec!["SET", "anotherkey", "will expire in a minute", "EX", "60"],
    )
    .await;
    read_assert_eq(&mut stream, b"+OK\r\n").await;
}

#[tokio::test]
async fn setex() {
    let mut stream = start_server().await;

    write_cmd(&mut stream, vec!["SETEX", "mykey", "90", "Hello"]).await;
    read_assert_eq(&mut stream, b"+OK\r\n").await;

    write_cmd(&mut stream, vec!["TTL", "mykey"]).await;
    read_assert_eq(&mut stream, b":90\r\n").await;

    write_cmd(&mut stream, vec!["GET", "mykey"]).await;
    read_assert_eq(&mut stream, b"+Hello\r\n").await;
}
