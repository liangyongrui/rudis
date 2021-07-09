//! Provides a type representing a Redis protocol frame as well as utilities for
//! parsing frames from a byte array.
//!
//! 目前使用的是 RESP2
//! todo 支持 RESP3

use std::{fmt, num::TryFromIntError, string::FromUtf8Error};

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

#[derive(Debug)]
pub enum Error {
    /// Not enough data is available to parse a message
    Incomplete,

    /// Invalid message encoding
    Other(crate::Error),
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

impl From<String> for Error {
    fn from(src: String) -> Error {
        Error::Other(src.into())
    }
}

impl From<&str> for Error {
    fn from(src: &str) -> Error {
        src.to_string().into()
    }
}

impl From<FromUtf8Error> for Error {
    fn from(_src: FromUtf8Error) -> Error {
        "protocol error; invalid frame format".into()
    }
}

impl From<TryFromIntError> for Error {
    fn from(_src: TryFromIntError) -> Error {
        "protocol error; invalid frame format".into()
    }
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Incomplete => "stream ended early".fmt(fmt),
            Error::Other(err) => err.fmt(fmt),
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
