//! Provides a type representing a Redis protocol frame as well as utilities for
//! parsing frames from a byte array.
//!
//! 目前使用的是 RESP2
//! todo 支持 RESP3

use std::{fmt, vec};

use bytes::Bytes;
use nom::{
    branch::alt,
    bytes::streaming::{tag, take_while, take_while1, take_while_m_n},
    combinator::map,
    sequence::delimited,
};

use crate::{
    slot::data_type::SimpleType,
    utils::{u8_to_i64, u8_to_string},
};

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

impl From<&SimpleType> for Frame {
    fn from(st: &SimpleType) -> Self {
        match st {
            SimpleType::Big => Frame::Null,
            SimpleType::String(s) => Frame::Simple(s.to_string()),
            SimpleType::Bytes(b) => Frame::Bulk(Bytes::copy_from_slice(&b)),
            SimpleType::Integer(i) => Frame::Integer(*i),
            SimpleType::Float(f) => Frame::Simple(format!("{}", f.0)),
            SimpleType::Null => Frame::Null,
        }
    }
}
impl PartialEq<&str> for Frame {
    fn eq(&self, other: &&str) -> bool {
        match self {
            Frame::Simple(s) => s.eq(other),
            Frame::Bulk(s) => s.eq(other),
            _ => false,
        }
    }
}

impl fmt::Display for Frame {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use std::str;

        match self {
            Frame::Simple(response) => response.fmt(fmt),
            Frame::Error(msg) => write!(fmt, "error: {}", msg),
            Frame::Integer(num) => num.fmt(fmt),
            Frame::Bulk(msg) => match str::from_utf8(msg) {
                Ok(string) => string.fmt(fmt),
                Err(_) => write!(fmt, "{:?}", msg),
            },
            Frame::Null => "(nil)".fmt(fmt),
            Frame::Array(parts) => {
                for (i, part) in parts.iter().enumerate() {
                    if i > 0 {
                        write!(fmt, " ")?;
                        part.fmt(fmt)?;
                    }
                }

                Ok(())
            }
        }
    }
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

impl From<Frame> for Vec<u8> {
    fn from(frame: Frame) -> Self {
        let mut res = vec![];
        match frame {
            Frame::Simple(a) => {
                res.push(b'+');
                res.extend_from_slice(a.as_bytes());
                res.extend_from_slice(b"\r\n");
            }
            Frame::Error(a) => {
                res.push(b'-');
                res.extend_from_slice(a.as_bytes());
                res.extend_from_slice(b"\r\n");
            }
            Frame::Integer(a) => {
                res.push(b':');
                res.extend_from_slice(a.to_string().as_bytes());
                res.extend_from_slice(b"\r\n");
            }
            Frame::Bulk(b) => {
                res.push(b'$');
                res.extend_from_slice(b.len().to_string().as_bytes());
                res.extend_from_slice(b"\r\n");
                res.extend_from_slice(&b[..]);
                res.extend_from_slice(b"\r\n");
            }
            Frame::Null => res.extend_from_slice(b"$-1\r\n"),
            Frame::Array(a) => {
                res.push(b'*');
                res.extend_from_slice(a.len().to_string().as_bytes());
                res.extend_from_slice(b"\r\n");
                for v in a {
                    res.append(&mut v.into());
                }
            }
        }
        res
    }
}
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let s = "*2\r\n*3\r\n:1\r\n$5\r\nhello\r\n:2\r\n+abc\r\n";
        let (_, f) = parse(s.as_bytes()).unwrap();
        let t = Frame::Array(vec![
            Frame::Array(vec![
                Frame::Integer(1),
                Frame::Bulk(Bytes::from_static(b"hello")),
                Frame::Integer(2),
            ]),
            Frame::Simple("abc".to_owned()),
        ]);
        assert_eq!(t, f);
        let v: Vec<u8> = f.into();
        assert_eq!(&v[..], s.as_bytes());
    }

    #[test]
    fn test2() {
        let hello = "hello".to_owned();
        let world = "world".to_owned();
        let s = format!(
            "*3\r\n$3\r\nSET\r\n${}\r\n{}\r\n${}\r\n{}\r\n",
            hello.len(),
            hello,
            world.len(),
            world
        );
        let b = s.as_bytes();
        let raw = Frame::Array(vec![
            Frame::Bulk("SET".into()),
            Frame::Bulk(hello.into()),
            Frame::Bulk(world.into()),
        ]);
        let set: Vec<u8> = raw.clone().into();
        assert_eq!(&set[..], b);
        let (_, f) = parse(s.as_bytes()).unwrap();
        assert_eq!(raw, f);
    }

    #[test]
    fn test3() {
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
