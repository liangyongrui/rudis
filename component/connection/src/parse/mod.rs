pub mod frame;

use std::{fmt, str, vec};

use keys::Key;

use crate::Frame;

/// Utility for parsing a command
///
/// Commands are represented as array frames. Each entry in the frame is a
/// "token". A `Parse` is initialized with the array frame and provides a
/// cursor-like API. Each command struct includes a `parse_frame` method that
/// uses a `Parse` to extract its fields.
#[derive(Debug)]
pub struct Parse<'a> {
    /// Array frame iterator.
    parts: vec::IntoIter<Frame<'a>>,
}
#[allow(clippy::module_name_repetitions)]
/// Error encountered while parsing a frame.
///
/// Only `EndOfStream` errors are handled at runtime. All other errors result in
/// the connection being terminated.
#[derive(Debug)]
pub enum ParseError {
    /// Attempting to extract a value failed due to the frame being fully
    /// consumed.
    EndOfStream,

    /// All other errors
    Other(common::Error),
}

impl<'a> Parse<'a> {
    /// Create a new `Parse` to parse the contents of `frame`.
    ///
    /// # Errors
    /// if `frame` is not an array frame (or Ping).
    pub fn new(frame: Frame<'a>) -> Result<Self, ParseError> {
        let array = match frame {
            Frame::Array(array) => array,
            Frame::Ping => vec![Frame::Ping],
            frame => return Err(format!("protocol error; expected array, got {:?}", frame).into()),
        };

        Ok(Parse {
            parts: array.into_iter(),
        })
    }

    /// Return the next entry. Array frames are arrays of frames, so the next
    /// entry is a frame.
    ///
    /// # Errors
    /// `EndOfStream`
    pub fn next_frame(&mut self) -> Result<Frame<'a>, ParseError> {
        self.parts.next().ok_or(ParseError::EndOfStream)
    }

    /// next key
    ///
    /// # Errors
    /// 1. `EndOfStream`
    /// 1. not bytes
    #[inline]
    pub fn next_key(&mut self) -> Result<Key, ParseError> {
        self.next_bulk().map(|t| t.into())
    }

    /// next bulk
    ///
    /// # Errors
    /// 1. `EndOfStream`
    /// 1. not bytes
    pub fn next_bulk(&mut self) -> Result<Box<[u8]>, ParseError> {
        match self.next_frame()? {
            Frame::Bulk(b) | Frame::Simple(b) => Ok(b.into()),
            frame => Err(format!("protocol error; got {:?}", frame).into()),
        }
    }

    /// Return the next entry as a string.
    ///
    /// # Errors
    /// the next entry cannot be represented as a String
    pub fn next_string(&mut self) -> Result<String, ParseError> {
        match self.next_frame()? {
            // Both `Simple` and `Bulk` representation may be strings. Strings
            // are parsed to UTF-8.
            //
            // While errors are stored as strings, they are considered separate
            // types.
            Frame::Bulk(data) | Frame::Simple(data) => str::from_utf8(data)
                .map(std::string::ToString::to_string)
                .map_err(|_| "protocol error; invalid string".into()),
            Frame::Ping => Ok("PING".to_owned()),
            frame => Err(format!("protocol error; got {:?}", frame).into()),
        }
    }

    /// Return the next entry as an integer.
    ///
    /// This includes `Simple`, `Bulk`, and `Integer` frame types. `Simple` and
    /// `Bulk` frame types are parsed.
    ///
    /// # Errors
    /// If the next entry cannot be represented as an integer, then an error is
    /// returned.
    pub fn next_int(&mut self) -> Result<i64, ParseError> {
        use atoi::atoi;

        const INVALID: &str = "protocol error; invalid number";

        match self.next_frame()? {
            // An integer frame type is already stored as an integer.
            Frame::Integer(v) => Ok(v),
            // Simple and bulk frames must be parsed as integers. If the parsing
            // fails, an error is returned.
            Frame::Bulk(data) | Frame::Simple(data) => match atoi::<i64>(data) {
                Some(e) => Ok(e),
                None => Err(INVALID.into()),
            },
            frame => Err(format!("protocol error; expected int frame but got {:?}", frame).into()),
        }
    }

    /// Ensure there are no more entries in the array
    /// # Errors
    /// if has more entries
    pub fn finish(&mut self) -> Result<(), ParseError> {
        if self.parts.next().is_none() {
            Ok(())
        } else {
            Err("protocol error; expected end of frame, but there was more".into())
        }
    }
}

impl From<String> for ParseError {
    fn from(src: String) -> ParseError {
        ParseError::Other(src.into())
    }
}

impl From<&str> for ParseError {
    fn from(src: &str) -> ParseError {
        src.to_string().into()
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::EndOfStream => "protocol error; unexpected end of stream".fmt(f),
            ParseError::Other(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for ParseError {}
