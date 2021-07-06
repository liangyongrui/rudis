use bytes::Bytes;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1, take_while_m_n},
    combinator::map,
    sequence::delimited,
};

use crate::utils::{u8_to_i64, u8_to_string};

/// A frame in the Redis protocol.
///
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Frame {
    Simple(String),
    Error(String),
    Integer(i64),
    Bulk(Bytes),
    Null,
    Array(Vec<Frame>),
}

fn parse_simple(i: &[u8]) -> nom::IResult<&[u8], Frame> {
    let (i, resp) = delimited(
        tag(b"+"),
        take_while(|c| c != b'\r' && c != b'\n'),
        tag(b"\r\n"),
    )(i)?;
    Ok((i, Frame::Simple(u8_to_string(resp))))
}

fn parse_error(i: &[u8]) -> nom::IResult<&[u8], Frame> {
    let (i, resp) = delimited(
        tag(b"-"),
        take_while1(|c| c != b'\r' && c != b'\n'),
        tag(b"\r\n"),
    )(i)?;
    Ok((i, Frame::Error(u8_to_string(resp))))
}

fn parse_int(i: &[u8]) -> nom::IResult<&[u8], Frame> {
    let (i, int) = delimited(
        tag(":"),
        map(take_while1(|c| c != b'\r' && c != b'\n'), |int: &[u8]| {
            u8_to_i64(int)
        }),
        tag(b"\r\n"),
    )(i)?;
    Ok((i, Frame::Integer(int)))
}

fn parse_bulk(i: &[u8]) -> nom::IResult<&[u8], Frame> {
    let (i, _) = tag("$")(i)?;
    let (i, len) = map(take_while1(|c| c != b'\r' && c != b'\n'), |int| {
        u8_to_i64(int)
    })(i)?;
    let (i, _) = tag(b"\r\n")(i)?;
    if len < 0 {
        Ok((i, Frame::Null))
    } else {
        let len = len as usize;
        let (i, data) = take_while_m_n(len, len, |_| true)(i)?;
        let (i, _) = tag(b"\r\n")(i)?;
        Ok((i, Frame::Bulk(Bytes::copy_from_slice(data))))
    }
}

fn parse_array(i: &[u8]) -> nom::IResult<&[u8], Frame> {
    let (i, _) = tag("*")(i)?;
    let (i, len) = map(take_while1(|c| c != b'\r' && c != b'\n'), |int| {
        u8_to_i64(int)
    })(i)?;
    let (mut i, _) = tag(b"\r\n")(i)?;
    if len < 0 {
        Ok((i, Frame::Null))
    } else {
        let mut res = vec![];
        for _ in 0..len {
            let (ni, f) = parse(i)?;
            res.push(f);
            i = ni
        }
        Ok((i, Frame::Array(res)))
    }
}

pub fn parse(i: &[u8]) -> nom::IResult<&[u8], Frame> {
    alt((
        parse_simple,
        parse_error,
        parse_int,
        parse_bulk,
        parse_array,
    ))(i)
}

mod test {
    use super::*;

    #[test]
    fn test() {
        let s = "*2\r\n*3\r\n:1\r\n$5\r\nhello\r\n:2\r\n+abc\r\n";
        let (_, f) = parse(s.as_bytes()).unwrap();
        assert_eq!(
            Frame::Array(vec![
                Frame::Array(vec![
                    Frame::Integer(1),
                    Frame::Bulk(Bytes::from_static(b"hello")),
                    Frame::Integer(2),
                ]),
                Frame::Simple("abc".to_owned()),
            ]),
            f
        )
    }

    #[test]
    fn test2() {
        fn f(x: i32) -> i32 {
            x + 1
        }
        let a = 1;
        for _ in 0..10 {
            let a = f(a);
            dbg!(a);
        }
    }
}
