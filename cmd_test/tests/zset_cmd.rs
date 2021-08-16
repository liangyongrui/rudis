//! 测试redis官网的demo

use ::server::Frame;
use cmd_test::{read_assert_eq, start_server, write_cmd};

#[tokio::test]
async fn zadd() {
    let mut stream = start_server().await;

    write_cmd(&mut stream, vec!["ZADD", "myzset", "1", "one"]).await;
    read_assert_eq(&mut stream, b":1\r\n").await;

    write_cmd(&mut stream, vec!["ZADD", "myzset", "1", "uno"]).await;
    read_assert_eq(&mut stream, b":1\r\n").await;

    write_cmd(
        &mut stream,
        vec!["ZADD", "myzset", "2", "two", "3", "three"],
    )
    .await;
    read_assert_eq(&mut stream, b":2\r\n").await;

    write_cmd(
        &mut stream,
        vec!["ZRANGE", "myzset", "0", "-1", "WITHSCORES"],
    )
    .await;
    let res: Vec<u8> = (&Frame::Array(vec![
        Frame::Simple("one".into()),
        Frame::Simple("1".into()),
        Frame::Simple("uno".into()),
        Frame::Simple("1".into()),
        Frame::Simple("two".into()),
        Frame::Simple("2".into()),
        Frame::Simple("three".into()),
        Frame::Simple("3".into()),
    ]))
        .into();
    read_assert_eq(&mut stream, &res).await;
}

#[tokio::test]
async fn zrange() {
    let mut stream = start_server().await;

    write_cmd(&mut stream, vec!["ZADD", "myzset", "1", "one"]).await;
    read_assert_eq(&mut stream, b":1\r\n").await;

    write_cmd(&mut stream, vec!["ZADD", "myzset", "2", "two"]).await;
    read_assert_eq(&mut stream, b":1\r\n").await;

    write_cmd(&mut stream, vec!["ZADD", "myzset", "3", "three"]).await;
    read_assert_eq(&mut stream, b":1\r\n").await;

    write_cmd(&mut stream, vec!["ZRANGE", "myzset", "0", "-1"]).await;
    let res: Vec<u8> = (&Frame::Array(vec![
        Frame::Simple("one".into()),
        Frame::Simple("two".into()),
        Frame::Simple("three".into()),
    ]))
        .into();
    read_assert_eq(&mut stream, &res).await;

    write_cmd(&mut stream, vec!["ZRANGE", "myzset", "2", "3"]).await;
    let res: Vec<u8> = (&Frame::Array(vec![Frame::Simple("three".into())])).into();
    read_assert_eq(&mut stream, &res).await;

    write_cmd(&mut stream, vec!["ZRANGE", "myzset", "-2", "-1"]).await;
    let res: Vec<u8> = (&Frame::Array(vec![
        Frame::Simple("two".into()),
        Frame::Simple("three".into()),
    ]))
        .into();
    read_assert_eq(&mut stream, &res).await;

    write_cmd(
        &mut stream,
        vec!["ZRANGE", "myzset", "0", "1", "WITHSCORES"],
    )
    .await;
    let res: Vec<u8> = (&Frame::Array(vec![
        Frame::Simple("one".into()),
        Frame::Simple("1".into()),
        Frame::Simple("two".into()),
        Frame::Simple("2".into()),
    ]))
        .into();
    read_assert_eq(&mut stream, &res).await;

    write_cmd(
        &mut stream,
        vec![
            "ZRANGE", "myzset", "(1", "+inf", "BYSCORE", "LIMIT", "1", "1",
        ],
    )
    .await;
    let res: Vec<u8> = (&Frame::Array(vec![Frame::Simple("three".into())])).into();
    read_assert_eq(&mut stream, &res).await;
}

#[tokio::test]
async fn zrangebylex() {
    let mut stream = start_server().await;

    write_cmd(
        &mut stream,
        vec![
            "ZADD", "myzset", "0", "a", "0", "b", "0", "c", "0", "d", "0", "e", "0", "f", "0", "g",
        ],
    )
    .await;
    read_assert_eq(&mut stream, b":7\r\n").await;

    write_cmd(&mut stream, vec!["ZRANGEBYLEX", "myzset", "-", "[c"]).await;
    let res: Vec<u8> = (&Frame::Array(vec![
        Frame::Simple("a".into()),
        Frame::Simple("b".into()),
        Frame::Simple("c".into()),
    ]))
        .into();
    read_assert_eq(&mut stream, &res).await;

    write_cmd(&mut stream, vec!["ZRANGEBYLEX", "myzset", "-", "(c"]).await;
    let res: Vec<u8> =
        (&Frame::Array(vec![Frame::Simple("a".into()), Frame::Simple("b".into())])).into();
    read_assert_eq(&mut stream, &res).await;

    write_cmd(&mut stream, vec!["ZRANGEBYLEX", "myzset", "[aaa", "(g"]).await;
    let res: Vec<u8> = (&Frame::Array(vec![
        Frame::Simple("b".into()),
        Frame::Simple("c".into()),
        Frame::Simple("d".into()),
        Frame::Simple("e".into()),
        Frame::Simple("f".into()),
    ]))
        .into();
    read_assert_eq(&mut stream, &res).await;
}

#[tokio::test]
async fn zrangebyscore() {
    let mut stream = start_server().await;

    write_cmd(&mut stream, vec!["ZADD", "myzset", "1", "one"]).await;
    read_assert_eq(&mut stream, b":1\r\n").await;

    write_cmd(&mut stream, vec!["ZADD", "myzset", "2", "two"]).await;
    read_assert_eq(&mut stream, b":1\r\n").await;

    write_cmd(&mut stream, vec!["ZADD", "myzset", "3", "three"]).await;
    read_assert_eq(&mut stream, b":1\r\n").await;

    write_cmd(&mut stream, vec!["ZRANGEBYSCORE", "myzset", "-inf", "+inf"]).await;
    let res: Vec<u8> = (&Frame::Array(vec![
        Frame::Simple("one".into()),
        Frame::Simple("two".into()),
        Frame::Simple("three".into()),
    ]))
        .into();
    read_assert_eq(&mut stream, &res).await;

    write_cmd(&mut stream, vec!["ZRANGEBYSCORE", "myzset", "1", "2"]).await;
    let res: Vec<u8> = (&Frame::Array(vec![
        Frame::Simple("one".into()),
        Frame::Simple("two".into()),
    ]))
        .into();
    read_assert_eq(&mut stream, &res).await;

    write_cmd(&mut stream, vec!["ZRANGEBYSCORE", "myzset", "(1", "2"]).await;
    let res: Vec<u8> = (&Frame::Array(vec![Frame::Simple("two".into())])).into();
    read_assert_eq(&mut stream, &res).await;

    write_cmd(&mut stream, vec!["ZRANGEBYSCORE", "myzset", "(1", "(2"]).await;
    let res: Vec<u8> = (&Frame::Array(vec![])).into();
    read_assert_eq(&mut stream, &res).await;
}

#[tokio::test]
async fn zrank() {
    let mut stream = start_server().await;

    write_cmd(&mut stream, vec!["ZADD", "myzset", "1", "one"]).await;
    read_assert_eq(&mut stream, b":1\r\n").await;

    write_cmd(&mut stream, vec!["ZADD", "myzset", "2", "two"]).await;
    read_assert_eq(&mut stream, b":1\r\n").await;

    write_cmd(&mut stream, vec!["ZADD", "myzset", "3", "three"]).await;
    read_assert_eq(&mut stream, b":1\r\n").await;

    write_cmd(&mut stream, vec!["ZRANK", "myzset", "three"]).await;
    read_assert_eq(&mut stream, b":2\r\n").await;

    write_cmd(&mut stream, vec!["ZRANK", "myzset", "four"]).await;
    read_assert_eq(&mut stream, b"$-1\r\n").await;
}

#[tokio::test]
async fn zrem() {
    let mut stream = start_server().await;

    write_cmd(&mut stream, vec!["ZADD", "myzset", "1", "one"]).await;
    read_assert_eq(&mut stream, b":1\r\n").await;

    write_cmd(&mut stream, vec!["ZADD", "myzset", "2", "two"]).await;
    read_assert_eq(&mut stream, b":1\r\n").await;

    write_cmd(&mut stream, vec!["ZADD", "myzset", "3", "three"]).await;
    read_assert_eq(&mut stream, b":1\r\n").await;

    write_cmd(&mut stream, vec!["ZREM", "myzset", "two"]).await;
    read_assert_eq(&mut stream, b":1\r\n").await;

    write_cmd(
        &mut stream,
        vec!["ZRANGE", "myzset", "0", "-1", "WITHSCORES"],
    )
    .await;
    let res: Vec<u8> = (&Frame::Array(vec![
        Frame::Simple("one".into()),
        Frame::Simple("1".into()),
        Frame::Simple("three".into()),
        Frame::Simple("3".into()),
    ]))
        .into();
    read_assert_eq(&mut stream, &res).await;
}

#[tokio::test]
async fn zremrangebylex() {
    let mut stream = start_server().await;

    write_cmd(
        &mut stream,
        vec![
            "ZADD", "myzset", "0", "aaaa", "0", "b", "0", "c", "0", "d", "0", "e",
        ],
    )
    .await;
    read_assert_eq(&mut stream, b":5\r\n").await;

    write_cmd(
        &mut stream,
        vec![
            "ZADD", "myzset", "0", "foo", "0", "zap", "0", "zip", "0", "ALPHA", "0", "alpha",
        ],
    )
    .await;
    read_assert_eq(&mut stream, b":5\r\n").await;

    write_cmd(&mut stream, vec!["ZRANGE", "myzset", "0", "-1"]).await;
    let res: Vec<u8> = (&Frame::Array(vec![
        Frame::Simple("ALPHA".into()),
        Frame::Simple("aaaa".into()),
        Frame::Simple("alpha".into()),
        Frame::Simple("b".into()),
        Frame::Simple("c".into()),
        Frame::Simple("d".into()),
        Frame::Simple("e".into()),
        Frame::Simple("foo".into()),
        Frame::Simple("zap".into()),
        Frame::Simple("zip".into()),
    ]))
        .into();
    read_assert_eq(&mut stream, &res).await;

    write_cmd(
        &mut stream,
        vec!["ZREMRANGEBYLEX", "myzset", "[alpha", "[omega"],
    )
    .await;
    read_assert_eq(&mut stream, b":6\r\n").await;

    write_cmd(&mut stream, vec!["ZRANGE", "myzset", "0", "-1"]).await;
    let res: Vec<u8> = (&Frame::Array(vec![
        Frame::Simple("ALPHA".into()),
        Frame::Simple("aaaa".into()),
        Frame::Simple("zap".into()),
        Frame::Simple("zip".into()),
    ]))
        .into();
    read_assert_eq(&mut stream, &res).await;
}

#[tokio::test]
async fn zremrangebyrank() {
    let mut stream = start_server().await;

    write_cmd(&mut stream, vec!["ZADD", "myzset", "1", "one"]).await;
    read_assert_eq(&mut stream, b":1\r\n").await;

    write_cmd(&mut stream, vec!["ZADD", "myzset", "2", "two"]).await;
    read_assert_eq(&mut stream, b":1\r\n").await;

    write_cmd(&mut stream, vec!["ZADD", "myzset", "3", "three"]).await;
    read_assert_eq(&mut stream, b":1\r\n").await;

    write_cmd(&mut stream, vec!["zremrangebyrank", "myzset", "0", "1"]).await;
    read_assert_eq(&mut stream, b":2\r\n").await;

    write_cmd(
        &mut stream,
        vec!["ZRANGE", "myzset", "0", "-1", "WITHSCORES"],
    )
    .await;
    let res: Vec<u8> = (&Frame::Array(vec![
        Frame::Simple("three".into()),
        Frame::Simple("3".into()),
    ]))
        .into();
    read_assert_eq(&mut stream, &res).await;
}

#[tokio::test]
async fn zremrangebyscore() {
    let mut stream = start_server().await;

    write_cmd(&mut stream, vec!["ZADD", "myzset", "1", "one"]).await;
    read_assert_eq(&mut stream, b":1\r\n").await;

    write_cmd(&mut stream, vec!["ZADD", "myzset", "2", "two"]).await;
    read_assert_eq(&mut stream, b":1\r\n").await;

    write_cmd(&mut stream, vec!["ZADD", "myzset", "3", "three"]).await;
    read_assert_eq(&mut stream, b":1\r\n").await;

    write_cmd(
        &mut stream,
        vec!["zremrangebyscore", "myzset", "-inf", "(2"],
    )
    .await;
    read_assert_eq(&mut stream, b":1\r\n").await;

    write_cmd(
        &mut stream,
        vec!["ZRANGE", "myzset", "0", "-1", "WITHSCORES"],
    )
    .await;
    let res: Vec<u8> = (&Frame::Array(vec![
        Frame::Simple("two".into()),
        Frame::Simple("2".into()),
        Frame::Simple("three".into()),
        Frame::Simple("3".into()),
    ]))
        .into();
    read_assert_eq(&mut stream, &res).await;
}

#[tokio::test]
async fn zrevrange() {
    let mut stream = start_server().await;

    write_cmd(&mut stream, vec!["ZADD", "myzset", "1", "one"]).await;
    read_assert_eq(&mut stream, b":1\r\n").await;

    write_cmd(&mut stream, vec!["ZADD", "myzset", "2", "two"]).await;
    read_assert_eq(&mut stream, b":1\r\n").await;

    write_cmd(&mut stream, vec!["ZADD", "myzset", "3", "three"]).await;
    read_assert_eq(&mut stream, b":1\r\n").await;

    write_cmd(&mut stream, vec!["zrevrange", "myzset", "0", "-1"]).await;
    let res: Vec<u8> = (&Frame::Array(vec![
        Frame::Simple("three".into()),
        Frame::Simple("two".into()),
        Frame::Simple("one".into()),
    ]))
        .into();
    read_assert_eq(&mut stream, &res).await;

    write_cmd(&mut stream, vec!["zrevrange", "myzset", "2", "3"]).await;
    let res: Vec<u8> = (&Frame::Array(vec![Frame::Simple("one".into())])).into();
    read_assert_eq(&mut stream, &res).await;

    write_cmd(&mut stream, vec!["zrevrange", "myzset", "-2", "-1"]).await;
    let res: Vec<u8> = (&Frame::Array(vec![
        Frame::Simple("two".into()),
        Frame::Simple("one".into()),
    ]))
        .into();
    read_assert_eq(&mut stream, &res).await;
}

#[tokio::test]
async fn zrevrangebylex() {
    let mut stream = start_server().await;

    write_cmd(
        &mut stream,
        vec![
            "ZADD", "myzset", "0", "a", "0", "b", "0", "c", "0", "d", "0", "e", "0", "f", "0", "g",
        ],
    )
    .await;
    read_assert_eq(&mut stream, b":7\r\n").await;

    write_cmd(&mut stream, vec!["ZrevRANGEBYLEX", "myzset", "[c", "-"]).await;
    let res: Vec<u8> = (&Frame::Array(vec![
        Frame::Simple("c".into()),
        Frame::Simple("b".into()),
        Frame::Simple("a".into()),
    ]))
        .into();
    read_assert_eq(&mut stream, &res).await;

    write_cmd(&mut stream, vec!["ZrevRANGEBYLEX", "myzset", "(c", "-"]).await;
    let res: Vec<u8> =
        (&Frame::Array(vec![Frame::Simple("b".into()), Frame::Simple("a".into())])).into();
    read_assert_eq(&mut stream, &res).await;

    write_cmd(&mut stream, vec!["ZrevRANGEBYLEX", "myzset", "(g", "[aaa"]).await;
    let res: Vec<u8> = (&Frame::Array(vec![
        Frame::Simple("f".into()),
        Frame::Simple("e".into()),
        Frame::Simple("d".into()),
        Frame::Simple("c".into()),
        Frame::Simple("b".into()),
    ]))
        .into();
    read_assert_eq(&mut stream, &res).await;
}

#[tokio::test]
async fn zrevrangebyscore() {
    let mut stream = start_server().await;

    write_cmd(&mut stream, vec!["ZADD", "myzset", "1", "one"]).await;
    read_assert_eq(&mut stream, b":1\r\n").await;

    write_cmd(&mut stream, vec!["ZADD", "myzset", "2", "two"]).await;
    read_assert_eq(&mut stream, b":1\r\n").await;

    write_cmd(&mut stream, vec!["ZADD", "myzset", "3", "three"]).await;
    read_assert_eq(&mut stream, b":1\r\n").await;

    write_cmd(
        &mut stream,
        vec!["zrevrangebyscore", "myzset", "+inf", "-inf"],
    )
    .await;
    let res: Vec<u8> = (&Frame::Array(vec![
        Frame::Simple("three".into()),
        Frame::Simple("two".into()),
        Frame::Simple("one".into()),
    ]))
        .into();
    read_assert_eq(&mut stream, &res).await;

    write_cmd(&mut stream, vec!["zrevrangebyscore", "myzset", "2", "1"]).await;
    let res: Vec<u8> = (&Frame::Array(vec![
        Frame::Simple("two".into()),
        Frame::Simple("one".into()),
    ]))
        .into();
    read_assert_eq(&mut stream, &res).await;

    write_cmd(&mut stream, vec!["zrevrangebyscore", "myzset", "2", "(1"]).await;
    let res: Vec<u8> = (&Frame::Array(vec![Frame::Simple("two".into())])).into();
    read_assert_eq(&mut stream, &res).await;

    write_cmd(&mut stream, vec!["zrevrangebyscore", "myzset", "(2", "(1"]).await;
    let res: Vec<u8> = (&Frame::Array(vec![])).into();
    read_assert_eq(&mut stream, &res).await;
}

#[tokio::test]
async fn zrevrank() {
    let mut stream = start_server().await;

    write_cmd(&mut stream, vec!["ZADD", "myzset", "1", "one"]).await;
    read_assert_eq(&mut stream, b":1\r\n").await;

    write_cmd(&mut stream, vec!["ZADD", "myzset", "2", "two"]).await;
    read_assert_eq(&mut stream, b":1\r\n").await;

    write_cmd(&mut stream, vec!["ZADD", "myzset", "3", "three"]).await;
    read_assert_eq(&mut stream, b":1\r\n").await;

    write_cmd(&mut stream, vec!["ZREVRANK", "myzset", "three"]).await;
    read_assert_eq(&mut stream, b":0\r\n").await;

    write_cmd(&mut stream, vec!["ZREVRANK", "myzset", "one"]).await;
    read_assert_eq(&mut stream, b":2\r\n").await;

    write_cmd(&mut stream, vec!["ZREVRANK", "myzset", "four"]).await;
    read_assert_eq(&mut stream, b"$-1\r\n").await;
}
