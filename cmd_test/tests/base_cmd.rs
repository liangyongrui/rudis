//! 测试redis官网的demo

use std::collections::HashMap;

use cmd_test::{read_assert_eq, start_server, write_cmd};

#[tokio::test]
async fn decr() {
    let mut stream = start_server().await;

    write_cmd(&mut stream, "GET decr_test").await;
    read_assert_eq(&mut stream, b"$-1\r\n").await;

    write_cmd(&mut stream, "DECR decr_test").await;
    read_assert_eq(&mut stream, b":-1\r\n").await;

    write_cmd(&mut stream, "GET decr_test").await;
    read_assert_eq(&mut stream, b":-1\r\n").await;

    write_cmd(&mut stream, "SET mykey 10").await;
    read_assert_eq(&mut stream, b"+OK\r\n").await;

    write_cmd(&mut stream, "DECR mykey").await;
    read_assert_eq(&mut stream, b":9\r\n").await;

    write_cmd(&mut stream, "SET mykey 234293482390480948029348230948").await;
    read_assert_eq(&mut stream, b"+OK\r\n").await;

    write_cmd(&mut stream, "DECR mykey").await;
    read_assert_eq(&mut stream, b"-number too large to fit in target type\r\n").await;
    dbg!("end1");

    write_cmd(&mut stream, "DECR mykey").await;
    read_assert_eq(&mut stream, b"-number too large to fit in target type\r\n").await;
    dbg!("end2");
}
#[test]
fn hash_map_test() {
    let mut map = HashMap::new();
    for i in 0..10000000i32 {
        map.insert(i, i + 1);
    }
    dbg!(map.capacity(), map.len());
}
