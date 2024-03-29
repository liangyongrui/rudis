pub mod frame;

use std::{cell::UnsafeCell, convert::TryInto, fmt, str, vec};

use keys::Key;

use self::frame::Frame;
use crate::float::Float;

/// Utility for parsing a command
///
/// Commands are represented as array frames. Each entry in the frame is a
/// "token". A `Parse` is initialized with the array frame and provides a
/// cursor-like API. Each command struct includes a `parse_frame` method that
/// uses a `Parse` to extract its fields.
#[derive(Debug)]
pub struct Parse<'a> {
    /// Array frame iterator.
    parts: UnsafeCell<vec::IntoIter<Frame<'a>>>,
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
    Other(crate::Error),
}

impl Clone for ParseError {
    #[inline]
    fn clone(&self) -> Self {
        match self {
            Self::EndOfStream => Self::EndOfStream,
            Self::Other(arg0) => Self::Other(arg0.to_string().into()),
        }
    }
}

impl<'a> Parse<'a> {
    /// Create a new `Parse` to parse the contents of `frame`.
    ///
    /// # Errors
    /// if `frame` is not an array frame (or Ping).
    #[inline]
    pub fn new(frame: Frame<'a>) -> Result<Self, ParseError> {
        let array = match frame {
            Frame::Array(array) => array,
            Frame::Ping => vec![Frame::Ping],
            frame => return Err(format!("protocol error; expected array, got {:?}", frame).into()),
        };

        Ok(Parse {
            parts: UnsafeCell::new(array.into_iter()),
        })
    }

    /// Return the next entry. Array frames are arrays of frames, so the next
    /// entry is a frame.
    ///
    /// # Errors
    /// `EndOfStream`
    #[inline]
    pub fn next_frame(&self) -> Result<Frame<'a>, ParseError> {
        unsafe { &mut *self.parts.get() }
            .next()
            .ok_or(ParseError::EndOfStream)
    }

    /// next key
    ///
    /// # Errors
    /// 1. `EndOfStream`
    /// 1. not bytes
    #[inline]
    pub fn next_key(&self) -> Result<Key, ParseError> {
        self.next_bulk().map(Into::into)
    }

    /// next bulk
    ///
    /// # Errors
    /// 1. `EndOfStream`
    /// 1. not bytes
    #[inline]
    pub fn next_bytes(&self) -> Result<&[u8], ParseError> {
        match self.next_frame()? {
            Frame::Bulk(b) | Frame::Simple(b) => Ok(b),
            frame => Err(format!("protocol error; got {:?}", frame).into()),
        }
    }

    /// next float
    ///
    /// # Errors
    /// 1. `EndOfStream`
    /// 1. not bytes
    #[inline]
    pub fn next_float(&self) -> Result<Float, ParseError> {
        Ok(self.next_frame()?.try_into()?)
    }

    /// next bulk
    ///
    /// # Errors
    /// 1. `EndOfStream`
    /// 1. not bytes
    #[inline]
    pub fn next_bulk(&self) -> Result<Box<[u8]>, ParseError> {
        match self.next_frame()? {
            Frame::Bulk(b) | Frame::Simple(b) => Ok(b.into()),
            frame => Err(format!("protocol error; got {:?}", frame).into()),
        }
    }

    /// Return the next entry as a string.
    ///
    /// # Errors
    /// the next entry cannot be represented as a String
    #[inline]
    pub fn next_str(&self) -> Result<&str, ParseError> {
        Ok(self.next_frame()?.try_into()?)
    }

    /// Return the next entry as a string.
    ///
    /// # Errors
    /// the next entry cannot be represented as a String
    #[inline]
    pub fn next_string(&self) -> Result<String, ParseError> {
        self.next_str().map(std::borrow::ToOwned::to_owned)
    }

    /// Return the next entry as an integer.
    ///
    /// This includes `Simple`, `Bulk`, and `Integer` frame types. `Simple` and
    /// `Bulk` frame types are parsed.
    ///
    /// # Errors
    /// If the next entry cannot be represented as an integer, then an error is
    /// returned.
    #[inline]
    pub fn next_int(&self) -> Result<i64, ParseError> {
        use atoi::atoi;

        const INVALID: &str = "protocol error; invalid number";

        match self.next_frame()? {
            // An integer frame type is already stored as an integer.
            Frame::Integer(v) => Ok(v),
            // Simple and bulk frames must be parsed as integers. If the parsing
            // fails, an error is returned.
            Frame::Bulk(data) | Frame::Simple(data) => {
                atoi::<i64>(data).map_or_else(|| Err(INVALID.into()), Ok)
            }
            frame => Err(format!("protocol error; expected int frame but got {:?}", frame).into()),
        }
    }

    /// Ensure there are no more entries in the array
    /// # Errors
    /// if has more entries
    #[inline]
    pub fn finish(&self) -> Result<(), ParseError> {
        if unsafe { &mut *self.parts.get() }.next().is_none() {
            Ok(())
        } else {
            Err("protocol error; expected end of frame, but there was more".into())
        }
    }
}
impl From<crate::Error> for ParseError {
    #[inline]
    fn from(e: crate::Error) -> Self {
        Self::Other(e)
    }
}
impl From<String> for ParseError {
    #[inline]
    fn from(src: String) -> Self {
        Self::Other(src.into())
    }
}

impl From<&str> for ParseError {
    #[inline]
    fn from(src: &str) -> Self {
        src.to_owned().into()
    }
}

impl fmt::Display for ParseError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EndOfStream => "protocol error; unexpected end of stream".fmt(f),
            Self::Other(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for ParseError {}
