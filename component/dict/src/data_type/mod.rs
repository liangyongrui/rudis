//! slot 每个value的数据类型
//!
//! 类型主要分为两种，简单类型 和 集合类型

use std::{convert::TryFrom, sync::Arc};
mod deque;
mod kvp;
mod set;
pub mod sorted_set;

pub use common::float::Float;
use keys::Key;
use serde::{Deserialize, Serialize};

pub use self::{deque::Deque, kvp::Kvp, set::Set, sorted_set::SortedSet};

/// slot value 的类型
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum DataType {
    Null,
    String(Arc<str>),
    Bytes(Key),
    Integer(i64),
    Float(Float),
    Kvp(Box<Kvp>),
    Deque(Deque),
    Set(Box<Set>),
    SortedSet(Box<SortedSet>),
}

impl From<&[u8]> for DataType {
    fn from(s: &[u8]) -> Self {
        DataType::Bytes(s.into())
    }
}
impl From<&str> for DataType {
    fn from(s: &str) -> Self {
        DataType::String(s.into())
    }
}
impl From<String> for DataType {
    fn from(s: String) -> Self {
        DataType::String(s.into())
    }
}
impl From<i64> for DataType {
    fn from(s: i64) -> Self {
        DataType::Integer(s)
    }
}

impl TryFrom<&DataType> for i64 {
    type Error = common::Error;

    fn try_from(value: &DataType) -> Result<Self, Self::Error> {
        let res = match value {
            DataType::String(s) => s.parse()?,
            DataType::Bytes(b) => std::str::from_utf8(b)?.parse()?,
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

    fn try_from(value: &DataType) -> Result<Self, Self::Error> {
        let res = match value {
            DataType::String(s) => s.as_ref().to_owned(),
            DataType::Bytes(b) => std::str::from_utf8(b)?.to_owned(),
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

    fn try_from(value: &DataType) -> Result<Self, Self::Error> {
        let res = match value {
            DataType::String(s) => s.to_string().parse()?,
            DataType::Bytes(b) => std::str::from_utf8(b)?.parse()?,
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
