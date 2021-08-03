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
    read_assert_eq(&mut stream, b":-1\r\n").await;

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
async fn get() {
    let mut stream = start_server().await;

    write_cmd(&mut stream, vec!["SET", "mykey", "Hello"]).await;
    read_assert_eq(&mut stream, b"+OK\r\n").await;

    write_cmd(&mut stream, vec!["GET", "mykey"]).await;
    read_assert_eq(&mut stream, b"$5\r\nHello\r\n").await;

    write_cmd(
        &mut stream,
        vec!["SET", "anotherkey", "will expire in a minute", "EX", "60"],
    )
    .await;
    read_assert_eq(&mut stream, b"+OK\r\n").await;
}
