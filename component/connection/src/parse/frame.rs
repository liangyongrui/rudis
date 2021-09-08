//! Provides a type representing a Redis protocol frame as well as utilities for
//! parsing frames from a byte array.

use std::{fmt, vec};

use nom::{
    branch::alt,
    bytes::streaming::{tag, take_while, take_while1, take_while_m_n},
    combinator::map,
    sequence::delimited,
};

/// A frame in the Redis protocol.
///
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Frame<'a> {
    Ping,
    Pong,
    Simple(&'a [u8]),
    OwnedSimple(Vec<u8>),
    OwnedStringSimple(String),
    Bulk(&'a [u8]),
    OwnedBulk(Vec<u8>),
    Error(&'a [u8]),
    OwnedError(String),
    Integer(i64),
    Null,
    Array(Vec<Frame<'a>>),
    /// not transfer
    NoRes,
}

impl Frame<'_> {
    #[inline]
    pub fn ok() -> Self {
        Frame::Simple(b"OK"[..].into())
    }
}

impl fmt::Display for Frame<'_> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use std::str;

        match self {
            Frame::Integer(num) => num.fmt(fmt),
            Frame::Bulk(msg) | Frame::Simple(msg) | Frame::Error(msg) => {
                match str::from_utf8(msg) {
                    Ok(string) => string.fmt(fmt),
                    Err(_) => write!(fmt, "{:?}", msg),
                }
            }
            Frame::OwnedBulk(msg) | Frame::OwnedSimple(msg) => match str::from_utf8(msg) {
                Ok(string) => string.fmt(fmt),
                Err(_) => write!(fmt, "{:?}", msg),
            },
            Frame::OwnedError(msg) | Frame::OwnedStringSimple(msg) => msg.fmt(fmt),
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
            Frame::NoRes => write!(fmt, "NoRes"),
        }
    }
}
impl fmt::Debug for Frame<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        std::fmt::Display::fmt(&self, f)
    }
}

#[inline]
fn parse_simple(i: &[u8]) -> nom::IResult<&[u8], Frame<'_>> {
    let (i, resp) = delimited(
        tag(b"+"),
        take_while(|c| c != b'\r' && c != b'\n'),
        tag(b"\r\n"),
    )(i)?;
    Ok((i, Frame::Simple(resp)))
}

#[inline]
fn parse_error(i: &[u8]) -> nom::IResult<&[u8], Frame<'_>> {
    let (i, resp) = delimited(
        tag(b"-"),
        take_while1(|c| c != b'\r' && c != b'\n'),
        tag(b"\r\n"),
    )(i)?;
    Ok((i, Frame::Error(resp)))
}

#[inline]
fn parse_int(i: &[u8]) -> nom::IResult<&[u8], Frame<'_>> {
    let (i, int) = delimited(
        tag(":"),
        map(take_while1(|c| c != b'\r' && c != b'\n'), |int: &[u8]| {
            atoi::atoi::<i64>(int).unwrap_or(0)
        }),
        tag(b"\r\n"),
    )(i)?;
    Ok((i, Frame::Integer(int)))
}

#[inline]
fn parse_bulk(i: &[u8]) -> nom::IResult<&[u8], Frame<'_>> {
    let (i, _) = tag("$")(i)?;
    let (i, len) = map(take_while1(|c| c != b'\r' && c != b'\n'), |int| {
        atoi::atoi::<i64>(int).unwrap_or(0)
    })(i)?;
    let (i, _) = tag(b"\r\n")(i)?;
    if len < 0 {
        Ok((i, Frame::Null))
    } else {
        let len = len as usize;
        let (i, data) = take_while_m_n(len, len, |_| true)(i)?;
        let (i, _) = tag(b"\r\n")(i)?;
        Ok((i, Frame::Bulk(data)))
    }
}

#[inline]
fn parse_array(i: &[u8]) -> nom::IResult<&[u8], Frame<'_>> {
    let (i, _) = tag("*")(i)?;
    let (i, len) = map(take_while1(|c| c != b'\r' && c != b'\n'), |int| {
        atoi::atoi::<i64>(int).unwrap_or(0)
    })(i)?;
    let (mut i, _) = tag(b"\r\n")(i)?;
    if len < 0 {
        Ok((i, Frame::Null))
    } else {
        let mut res = vec![];
        for _ in 0..len {
            let (ni, f) = parse(i)?;
            res.push(f);
            i = ni;
        }
        Ok((i, Frame::Array(res)))
    }
}

#[inline]
fn parse_ping(i: &[u8]) -> nom::IResult<&[u8], Frame<'_>> {
    let (i, _) = tag(b"PING\r\n")(i)?;
    Ok((i, Frame::Ping))
}

/// parse bytes to frame
///
/// # Errors
/// parse failed
#[inline]
fn parse(i: &[u8]) -> nom::IResult<&[u8], Frame<'_>> {
    alt((
        parse_simple,
        parse_error,
        parse_int,
        parse_bulk,
        parse_array,
        parse_ping,
    ))(i)
}

/// parse bytes to frame
///
/// # Errors
/// parse failed
#[inline]

pub fn parse_2(i: &[u8]) -> common::Result<Option<(usize, Frame<'_>)>> {
    let old_len = i.len();
    match parse(i) {
        Ok(o) => Ok(Some((old_len - o.0.len(), o.1))),
        Err(nom::Err::Incomplete(_)) => Ok(None),
        Err(e) => return Err(format!("parse failed, {:?}", e).into()),
    }
}

impl Frame<'_> {
    pub fn write(&self, res: &mut Vec<u8>) {
        match self {
            Frame::Simple(a) => {
                res.push(b'+');
                res.extend_from_slice(a);
                res.extend_from_slice(b"\r\n");
            }
            Frame::OwnedSimple(a) => {
                res.push(b'+');
                res.extend_from_slice(a);
                res.extend_from_slice(b"\r\n");
            }
            Frame::OwnedStringSimple(a) => {
                res.push(b'+');
                res.extend_from_slice(a.as_bytes());
                res.extend_from_slice(b"\r\n");
            }
            Frame::Error(a) => {
                res.push(b'-');
                res.extend_from_slice(a);
                res.extend_from_slice(b"\r\n");
            }
            Frame::OwnedError(a) => {
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
            Frame::OwnedBulk(b) => {
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
            Frame::NoRes => {}
        }
    }
}

impl From<&Frame<'_>> for Vec<u8> {
    fn from(frame: &Frame) -> Self {
        if let Frame::NoRes = frame {
            return vec![];
        }
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
            Frame::Simple(b"abc"[..].into()),
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
            Frame::Bulk(hello.as_bytes()),
            Frame::Bulk(world.as_bytes()),
        ]);
        let set: Vec<u8> = (&raw).into();
        assert_eq!(&set[..], b);
        let (_, f) = parse(s.as_bytes()).unwrap();
        assert_eq!(raw, f);
    }
}
