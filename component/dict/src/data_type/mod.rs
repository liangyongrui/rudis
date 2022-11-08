//! slot 每个value的数据类型
//!
//! 类型主要分为两种，简单类型 和 集合类型

use std::convert::TryFrom;
mod deque;
mod kvp;
mod set;
pub mod sorted_set;

pub use common::float::Float;
use serde::{Deserialize, Serialize};

pub use self::{deque::Deque, kvp::Kvp, set::Set, sorted_set::SortedSet};

/// slot value 的类型
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum DataType {
    Null,
    String(Vec<u8>),
    Bytes(Vec<u8>),
    Integer(i64),
    Float(Float),
    Kvp(Box<Kvp>),
    Deque(Deque),
    Set(Box<Set>),
    SortedSet(Box<SortedSet>),
}

impl From<&[u8]> for DataType {
    #[inline]
    fn from(s: &[u8]) -> Self {
        DataType::Bytes(s.into())
    }
}
impl From<&str> for DataType {
    #[inline]
    fn from(s: &str) -> Self {
        DataType::String(s.as_bytes().into())
    }
}
impl From<String> for DataType {
    #[inline]
    fn from(s: String) -> Self {
        DataType::String(s.as_bytes().into())
    }
}
impl From<i64> for DataType {
    #[inline]
    fn from(s: i64) -> Self {
        DataType::Integer(s)
    }
}

impl TryFrom<&DataType> for i64 {
    type Error = common::Error;

    #[inline]
    fn try_from(value: &DataType) -> Result<Self, Self::Error> {
        let res = match value {
            DataType::String(b) | DataType::Bytes(b) => atoi::atoi::<i64>(b)
                .ok_or_else::<Self::Error, _>(|| {
                    "value is not an integer or out of range".into()
                })?,
            DataType::Integer(i) => *i,
            _ => {
                return Err(
                    "WRONGTYPE Operation against a key holding the wrong kind of value".into(),
                )
            }
        };
        Ok(res)
    }
}

impl TryFrom<&DataType> for String {
    type Error = common::Error;

    #[inline]
    fn try_from(value: &DataType) -> Result<Self, Self::Error> {
        let res = match value {
            DataType::String(b) | DataType::Bytes(b) => std::str::from_utf8(b)?.to_owned(),
            DataType::Integer(i) => format!("{}", i),
            DataType::Float(f) => format!("{}", f.0),
            _ => {
                return Err(
                    "WRONGTYPE Operation against a key holding the wrong kind of value".into(),
                )
            }
        };
        Ok(res)
    }
}

impl TryFrom<&DataType> for f64 {
    type Error = common::Error;

    #[inline]
    fn try_from(value: &DataType) -> Result<Self, Self::Error> {
        let res = match value {
            DataType::String(b) | DataType::Bytes(b) => std::str::from_utf8(b)?.parse()?,
            #[allow(clippy::cast_precision_loss)]
            DataType::Integer(i) => *i as _,
            DataType::Float(f) => f.0,
            _ => {
                return Err(
                    "WRONGTYPE Operation against a key holding the wrong kind of value".into(),
                )
            }
        };
        Ok(res)
    }
}
