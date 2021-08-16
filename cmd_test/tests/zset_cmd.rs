//! 测试redis官网的demo

use ::server::Frame;
use cmd_test::{next_frame_eq, start_server, write_cmd};

#[tokio::test]
async fn zadd() {
    let mut connection = start_server().await;

    write_cmd(&mut connection.stream, vec!["ZADD", "myzset", "1", "one"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["ZADD", "myzset", "1", "uno"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(
        &mut connection.stream,
        vec!["ZADD", "myzset", "2", "two", "3", "three"],
    )
    .await;
    next_frame_eq(&mut connection, Frame::Integer(2)).await;

    write_cmd(
        &mut connection.stream,
        vec!["ZRANGE", "myzset", "0", "-1", "WITHSCORES"],
    )
    .await;
    let res = Frame::Array(vec![
        Frame::Simple("one".into()),
        Frame::Simple("1".into()),
        Frame::Simple("uno".into()),
        Frame::Simple("1".into()),
        Frame::Simple("two".into()),
        Frame::Simple("2".into()),
        Frame::Simple("three".into()),
        Frame::Simple("3".into()),
    ]);
    next_frame_eq(&mut connection, res).await;
}

#[tokio::test]
async fn zrange() {
    let mut connection = start_server().await;

    write_cmd(&mut connection.stream, vec!["ZADD", "myzset", "1", "one"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["ZADD", "myzset", "2", "two"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["ZADD", "myzset", "3", "three"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["ZRANGE", "myzset", "0", "-1"]).await;
    let res = Frame::Array(vec![
        Frame::Simple("one".into()),
        Frame::Simple("two".into()),
        Frame::Simple("three".into()),
    ]);
    next_frame_eq(&mut connection, res).await;

    write_cmd(&mut connection.stream, vec!["ZRANGE", "myzset", "2", "3"]).await;
    let res = Frame::Array(vec![Frame::Simple("three".into())]);
    next_frame_eq(&mut connection, res).await;

    write_cmd(&mut connection.stream, vec!["ZRANGE", "myzset", "-2", "-1"]).await;
    let res = Frame::Array(vec![
        Frame::Simple("two".into()),
        Frame::Simple("three".into()),
    ]);
    next_frame_eq(&mut connection, res).await;

    write_cmd(
        &mut connection.stream,
        vec!["ZRANGE", "myzset", "0", "1", "WITHSCORES"],
    )
    .await;
    let res = Frame::Array(vec![
        Frame::Simple("one".into()),
        Frame::Simple("1".into()),
        Frame::Simple("two".into()),
        Frame::Simple("2".into()),
    ]);
    next_frame_eq(&mut connection, res).await;

    write_cmd(
        &mut connection.stream,
        vec![
            "ZRANGE", "myzset", "(1", "+inf", "BYSCORE", "LIMIT", "1", "1",
        ],
    )
    .await;
    let res = Frame::Array(vec![Frame::Simple("three".into())]);
    next_frame_eq(&mut connection, res).await;
}

#[tokio::test]
async fn zrangebylex() {
    let mut connection = start_server().await;

    write_cmd(
        &mut connection.stream,
        vec![
            "ZADD", "myzset", "0", "a", "0", "b", "0", "c", "0", "d", "0", "e", "0", "f", "0", "g",
        ],
    )
    .await;
    next_frame_eq(&mut connection, Frame::Integer(7)).await;

    write_cmd(
        &mut connection.stream,
        vec!["ZRANGEBYLEX", "myzset", "-", "[c"],
    )
    .await;
    let res = Frame::Array(vec![
        Frame::Simple("a".into()),
        Frame::Simple("b".into()),
        Frame::Simple("c".into()),
    ]);
    next_frame_eq(&mut connection, res).await;

    write_cmd(
        &mut connection.stream,
        vec!["ZRANGEBYLEX", "myzset", "-", "(c"],
    )
    .await;
    let res = Frame::Array(vec![Frame::Simple("a".into()), Frame::Simple("b".into())]);
    next_frame_eq(&mut connection, res).await;

    write_cmd(
        &mut connection.stream,
        vec!["ZRANGEBYLEX", "myzset", "[aaa", "(g"],
    )
    .await;
    let res = Frame::Array(vec![
        Frame::Simple("b".into()),
        Frame::Simple("c".into()),
        Frame::Simple("d".into()),
        Frame::Simple("e".into()),
        Frame::Simple("f".into()),
    ]);
    next_frame_eq(&mut connection, res).await;
}

#[tokio::test]
async fn zrangebyscore() {
    let mut connection = start_server().await;

    write_cmd(&mut connection.stream, vec!["ZADD", "myzset", "1", "one"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["ZADD", "myzset", "2", "two"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["ZADD", "myzset", "3", "three"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(
        &mut connection.stream,
        vec!["ZRANGEBYSCORE", "myzset", "-inf", "+inf"],
    )
    .await;
    let res = Frame::Array(vec![
        Frame::Simple("one".into()),
        Frame::Simple("two".into()),
        Frame::Simple("three".into()),
    ]);
    next_frame_eq(&mut connection, res).await;

    write_cmd(
        &mut connection.stream,
        vec!["ZRANGEBYSCORE", "myzset", "1", "2"],
    )
    .await;
    let res = Frame::Array(vec![
        Frame::Simple("one".into()),
        Frame::Simple("two".into()),
    ]);
    next_frame_eq(&mut connection, res).await;

    write_cmd(
        &mut connection.stream,
        vec!["ZRANGEBYSCORE", "myzset", "(1", "2"],
    )
    .await;
    let res = Frame::Array(vec![Frame::Simple("two".into())]);
    next_frame_eq(&mut connection, res).await;

    write_cmd(
        &mut connection.stream,
        vec!["ZRANGEBYSCORE", "myzset", "(1", "(2"],
    )
    .await;
    let res = Frame::Array(vec![]);
    next_frame_eq(&mut connection, res).await;
}

#[tokio::test]
async fn zrank() {
    let mut connection = start_server().await;

    write_cmd(&mut connection.stream, vec!["ZADD", "myzset", "1", "one"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["ZADD", "myzset", "2", "two"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["ZADD", "myzset", "3", "three"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["ZRANK", "myzset", "three"]).await;
    next_frame_eq(&mut connection, Frame::Integer(2)).await;

    write_cmd(&mut connection.stream, vec!["ZRANK", "myzset", "four"]).await;
    next_frame_eq(&mut connection, Frame::Null).await;
}

#[tokio::test]
async fn zrem() {
    let mut connection = start_server().await;

    write_cmd(&mut connection.stream, vec!["ZADD", "myzset", "1", "one"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["ZADD", "myzset", "2", "two"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["ZADD", "myzset", "3", "three"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["ZREM", "myzset", "two"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(
        &mut connection.stream,
        vec!["ZRANGE", "myzset", "0", "-1", "WITHSCORES"],
    )
    .await;
    let res = Frame::Array(vec![
        Frame::Simple("one".into()),
        Frame::Simple("1".into()),
        Frame::Simple("three".into()),
        Frame::Simple("3".into()),
    ]);
    next_frame_eq(&mut connection, res).await;
}

#[tokio::test]
async fn zremrangebylex() {
    let mut connection = start_server().await;

    write_cmd(
        &mut connection.stream,
        vec![
            "ZADD", "myzset", "0", "aaaa", "0", "b", "0", "c", "0", "d", "0", "e",
        ],
    )
    .await;
    next_frame_eq(&mut connection, Frame::Integer(5)).await;

    write_cmd(
        &mut connection.stream,
        vec![
            "ZADD", "myzset", "0", "foo", "0", "zap", "0", "zip", "0", "ALPHA", "0", "alpha",
        ],
    )
    .await;
    next_frame_eq(&mut connection, Frame::Integer(5)).await;

    write_cmd(&mut connection.stream, vec!["ZRANGE", "myzset", "0", "-1"]).await;
    let res = Frame::Array(vec![
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
    ]);
    next_frame_eq(&mut connection, res).await;

    write_cmd(
        &mut connection.stream,
        vec!["ZREMRANGEBYLEX", "myzset", "[alpha", "[omega"],
    )
    .await;
    next_frame_eq(&mut connection, Frame::Integer(6)).await;

    write_cmd(&mut connection.stream, vec!["ZRANGE", "myzset", "0", "-1"]).await;
    let res = Frame::Array(vec![
        Frame::Simple("ALPHA".into()),
        Frame::Simple("aaaa".into()),
        Frame::Simple("zap".into()),
        Frame::Simple("zip".into()),
    ]);
    next_frame_eq(&mut connection, res).await;
}

#[tokio::test]
async fn zremrangebyrank() {
    let mut connection = start_server().await;

    write_cmd(&mut connection.stream, vec!["ZADD", "myzset", "1", "one"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["ZADD", "myzset", "2", "two"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["ZADD", "myzset", "3", "three"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(
        &mut connection.stream,
        vec!["zremrangebyrank", "myzset", "0", "1"],
    )
    .await;
    next_frame_eq(&mut connection, Frame::Integer(2)).await;

    write_cmd(
        &mut connection.stream,
        vec!["ZRANGE", "myzset", "0", "-1", "WITHSCORES"],
    )
    .await;
    let res = Frame::Array(vec![
        Frame::Simple("three".into()),
        Frame::Simple("3".into()),
    ]);
    next_frame_eq(&mut connection, res).await;
}

#[tokio::test]
async fn zremrangebyscore() {
    let mut connection = start_server().await;

    write_cmd(&mut connection.stream, vec!["ZADD", "myzset", "1", "one"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["ZADD", "myzset", "2", "two"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["ZADD", "myzset", "3", "three"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(
        &mut connection.stream,
        vec!["zremrangebyscore", "myzset", "-inf", "(2"],
    )
    .await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(
        &mut connection.stream,
        vec!["ZRANGE", "myzset", "0", "-1", "WITHSCORES"],
    )
    .await;
    let res = Frame::Array(vec![
        Frame::Simple("two".into()),
        Frame::Simple("2".into()),
        Frame::Simple("three".into()),
        Frame::Simple("3".into()),
    ]);
    next_frame_eq(&mut connection, res).await;
}

#[tokio::test]
async fn zrevrange() {
    let mut connection = start_server().await;

    write_cmd(&mut connection.stream, vec!["ZADD", "myzset", "1", "one"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["ZADD", "myzset", "2", "two"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["ZADD", "myzset", "3", "three"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(
        &mut connection.stream,
        vec!["zrevrange", "myzset", "0", "-1"],
    )
    .await;
    let res = Frame::Array(vec![
        Frame::Simple("three".into()),
        Frame::Simple("two".into()),
        Frame::Simple("one".into()),
    ]);
    next_frame_eq(&mut connection, res).await;

    write_cmd(
        &mut connection.stream,
        vec!["zrevrange", "myzset", "2", "3"],
    )
    .await;
    let res = Frame::Array(vec![Frame::Simple("one".into())]);
    next_frame_eq(&mut connection, res).await;

    write_cmd(
        &mut connection.stream,
        vec!["zrevrange", "myzset", "-2", "-1"],
    )
    .await;
    let res = Frame::Array(vec![
        Frame::Simple("two".into()),
        Frame::Simple("one".into()),
    ]);
    next_frame_eq(&mut connection, res).await;
}

#[tokio::test]
async fn zrevrangebylex() {
    let mut connection = start_server().await;

    write_cmd(
        &mut connection.stream,
        vec![
            "ZADD", "myzset", "0", "a", "0", "b", "0", "c", "0", "d", "0", "e", "0", "f", "0", "g",
        ],
    )
    .await;
    next_frame_eq(&mut connection, Frame::Integer(7)).await;

    write_cmd(
        &mut connection.stream,
        vec!["ZrevRANGEBYLEX", "myzset", "[c", "-"],
    )
    .await;
    let res = Frame::Array(vec![
        Frame::Simple("c".into()),
        Frame::Simple("b".into()),
        Frame::Simple("a".into()),
    ]);
    next_frame_eq(&mut connection, res).await;

    write_cmd(
        &mut connection.stream,
        vec!["ZrevRANGEBYLEX", "myzset", "(c", "-"],
    )
    .await;
    let res = Frame::Array(vec![Frame::Simple("b".into()), Frame::Simple("a".into())]);
    next_frame_eq(&mut connection, res).await;

    write_cmd(
        &mut connection.stream,
        vec!["ZrevRANGEBYLEX", "myzset", "(g", "[aaa"],
    )
    .await;
    let res = Frame::Array(vec![
        Frame::Simple("f".into()),
        Frame::Simple("e".into()),
        Frame::Simple("d".into()),
        Frame::Simple("c".into()),
        Frame::Simple("b".into()),
    ]);
    next_frame_eq(&mut connection, res).await;
}

#[tokio::test]
async fn zrevrangebyscore() {
    let mut connection = start_server().await;

    write_cmd(&mut connection.stream, vec!["ZADD", "myzset", "1", "one"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["ZADD", "myzset", "2", "two"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["ZADD", "myzset", "3", "three"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(
        &mut connection.stream,
        vec!["zrevrangebyscore", "myzset", "+inf", "-inf"],
    )
    .await;
    let res = Frame::Array(vec![
        Frame::Simple("three".into()),
        Frame::Simple("two".into()),
        Frame::Simple("one".into()),
    ]);
    next_frame_eq(&mut connection, res).await;

    write_cmd(
        &mut connection.stream,
        vec!["zrevrangebyscore", "myzset", "2", "1"],
    )
    .await;
    let res = Frame::Array(vec![
        Frame::Simple("two".into()),
        Frame::Simple("one".into()),
    ]);
    next_frame_eq(&mut connection, res).await;

    write_cmd(
        &mut connection.stream,
        vec!["zrevrangebyscore", "myzset", "2", "(1"],
    )
    .await;
    let res = Frame::Array(vec![Frame::Simple("two".into())]);
    next_frame_eq(&mut connection, res).await;

    write_cmd(
        &mut connection.stream,
        vec!["zrevrangebyscore", "myzset", "(2", "(1"],
    )
    .await;
    let res = Frame::Array(vec![]);
    next_frame_eq(&mut connection, res).await;
}

#[tokio::test]
async fn zrevrank() {
    let mut connection = start_server().await;

    write_cmd(&mut connection.stream, vec!["ZADD", "myzset", "1", "one"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["ZADD", "myzset", "2", "two"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["ZADD", "myzset", "3", "three"]).await;
    next_frame_eq(&mut connection, Frame::Integer(1)).await;

    write_cmd(&mut connection.stream, vec!["ZREVRANK", "myzset", "three"]).await;
    next_frame_eq(&mut connection, Frame::Integer(0)).await;

    write_cmd(&mut connection.stream, vec!["ZREVRANK", "myzset", "one"]).await;
    next_frame_eq(&mut connection, Frame::Integer(2)).await;

    write_cmd(&mut connection.stream, vec!["ZREVRANK", "myzset", "four"]).await;
    next_frame_eq(&mut connection, Frame::Null).await;
}
