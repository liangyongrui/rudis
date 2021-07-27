//! 每个命令简单测一下功能是否正常，无需关联
//! 命令功能的详细测试，在db模块测

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
}
