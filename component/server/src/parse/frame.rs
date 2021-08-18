//! Provides a type representing a Redis protocol frame as well as utilities for
//! parsing frames from a byte array.
//!
//! 目前使用的是 RESP2
//! todo 支持 RESP3

use std::{fmt, sync::Arc, vec};

use common::{u8_to_i64, u8_to_string};
use dict::data_type::DataType;
use nom::{
    branch::alt,
    bytes::streaming::{tag, take_while, take_while1, take_while_m_n},
    combinator::map,
    sequence::delimited,
};

/// A frame in the Redis protocol.
///
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Frame {
    Ping,
    Pong,
    Simple(Arc<str>),
    Error(String),
    Integer(i64),
    Bulk(Arc<[u8]>),
    Null,
    Array(Vec<Frame>),
}

impl Frame {
    pub fn ok() -> Self {
        Frame::Simple("OK".into())
    }
}

impl From<DataType> for Frame {
    fn from(st: DataType) -> Self {
        match st {
            DataType::String(s) => Frame::Simple(s),
            DataType::Bytes(b) => Frame::Bulk(b),
            DataType::Integer(i) => Frame::Integer(i),
            DataType::Float(f) => Frame::Simple(format!("{}", f.0).into()),
            DataType::Null => Frame::Null,
            _ => Frame::Error("type not support".into()),
        }
    }
}

impl From<&DataType> for Frame {
    fn from(st: &DataType) -> Self {
        match st {
            DataType::String(s) => Frame::Simple(s.clone()),
            DataType::Bytes(b) => Frame::Bulk(b.clone()),
            DataType::Integer(i) => Frame::Integer(*i),
            DataType::Float(f) => Frame::Simple(format!("{}", f.0).into()),
            DataType::Null => Frame::Null,
            _ => Frame::Error("type not support".into()),
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
            Frame::Ping => write!(fmt, "PING"),
            Frame::Pong => write!(fmt, "PONG"),
        }
    }
}
impl fmt::Debug for Frame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        std::fmt::Display::fmt(&self, f)
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
    Ok((
        i,
        Frame::Error(
            std::str::from_utf8(resp)
                .expect("protocol error; invalid string")
                .to_string(),
        ),
    ))
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
        Ok((i, Frame::Bulk(Arc::from(Box::from(data)))))
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

fn parse_ping(i: &[u8]) -> nom::IResult<&[u8], Frame> {
    let (i, _) = tag(b"PING\r\n")(i)?;
    Ok((i, Frame::Ping))
}
pub fn parse(i: &[u8]) -> nom::IResult<&[u8], Frame> {
    alt((
        parse_simple,
        parse_error,
        parse_int,
        parse_bulk,
        parse_array,
        parse_ping,
    ))(i)
}

impl Frame {
    pub fn write(&self, res: &mut Vec<u8>) {
        match self {
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
                    v.write(res);
                }
            }
            Frame::Ping => res.extend_from_slice(b"+PING\r\n"),
            Frame::Pong => res.extend_from_slice(b"+PONG\r\n"),
        }
    }
}

impl From<&Frame> for Vec<u8> {
    fn from(frame: &Frame) -> Self {
        let mut res = Vec::with_capacity(128);
        frame.write(&mut res);
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
                Frame::Bulk(b"hello"[..].into()),
                Frame::Integer(2),
            ]),
            Frame::Simple("abc".into()),
        ]);
        assert_eq!(t, f);
        let v: Vec<u8> = (&f).into();
        assert_eq!(&v[..], s.as_bytes());
    }

    #[test]
    fn test2() {
        let hello = "hello";
        let world = "world";
        let s = format!(
            "*3\r\n$3\r\nSET\r\n${}\r\n{}\r\n${}\r\n{}\r\n",
            hello.len(),
            hello,
            world.len(),
            world
        );
        let b = s.as_bytes();
        let raw = Frame::Array(vec![
            Frame::Bulk(b"SET"[..].into()),
            Frame::Bulk(hello.as_bytes().into()),
            Frame::Bulk(world.as_bytes().into()),
        ]);
        let set: Vec<u8> = (&raw).into();
        assert_eq!(&set[..], b);
        let (_, f) = parse(s.as_bytes()).unwrap();
        assert_eq!(raw, f);
    }
}