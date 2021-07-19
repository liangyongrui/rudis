mod blob;
mod hash;
mod integer;
mod list;
mod set;
mod sorted_set;

use std::convert::TryFrom;

use bytes::Bytes;
use nom::AsBytes;
use serde::{Deserialize, Serialize};
pub use sorted_set::{Node as SortedSetNode, RangeItem as ZrangeItem};

use self::{hash::Hash, list::List, set::Set, sorted_set::SortedSet};
use crate::Frame;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SimpleTypePair {
    pub key: SimpleType,
    pub value: SimpleType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataType {
    SimpleType(SimpleType),
    AggregateType(AggregateType),
}

impl PartialEq for DataType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (DataType::SimpleType(o1), DataType::SimpleType(o2)) => o1 == o2,
            _ => false,
        }
    }
}
impl Eq for DataType {}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SimpleType {
    Blob(Vec<u8>),
    SimpleString(String),
    Integer(i64),
    Null,
    // Bool(bool),
    // todo
    // VerbatimString,
    // todo
    // BigNumber,
}

impl PartialOrd for SimpleType {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (SimpleType::Blob(b1), SimpleType::Blob(b2)) => b1.partial_cmp(b2),
            (SimpleType::Blob(_), _) => Some(std::cmp::Ordering::Greater),
            (SimpleType::Null, SimpleType::Null) => Some(std::cmp::Ordering::Equal),
            (SimpleType::Null, _) => Some(std::cmp::Ordering::Less),
            (SimpleType::SimpleString(_), SimpleType::Blob(_)) => Some(std::cmp::Ordering::Less),
            (SimpleType::SimpleString(s1), SimpleType::SimpleString(s2)) => s1.partial_cmp(s2),
            (SimpleType::SimpleString(_), _) => Some(std::cmp::Ordering::Greater),
            (SimpleType::Integer(_), SimpleType::Null) => Some(std::cmp::Ordering::Greater),
            (SimpleType::Integer(i1), SimpleType::Integer(i2)) => i1.partial_cmp(i2),
            (SimpleType::Integer(_), _) => Some(std::cmp::Ordering::Less),
        }
    }
}

impl Ord for SimpleType {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregateType {
    List(List),
    Hash(Hash),
    Set(Set),
    SortedSet(SortedSet),
}

impl TryFrom<&SimpleType> for f64 {
    type Error = &'static str;

    fn try_from(value: &SimpleType) -> Result<Self, Self::Error> {
        match value {
            SimpleType::SimpleString(s) => s.parse().map_err(|_| "类型不对"),
            SimpleType::Blob(b) => {
                let s = std::str::from_utf8(b.as_bytes()).map_err(|_| "类型不对")?;

                s.parse().map_err(|_| "类型不对")
            }
            SimpleType::Integer(i) => Ok(*i as _),
            SimpleType::Null => Ok(0.0),
        }
    }
}
impl TryFrom<DataType> for SimpleType {
    type Error = &'static str;

    fn try_from(value: DataType) -> Result<Self, Self::Error> {
        match value {
            DataType::SimpleType(s) => Ok(s),
            _ => Err("类型不对"),
        }
    }
}

impl TryFrom<SimpleType> for String {
    type Error = &'static str;

    fn try_from(value: SimpleType) -> Result<Self, Self::Error> {
        match value {
            SimpleType::SimpleString(s) => Ok(s),
            _ => Err("类型不对"),
        }
    }
}
impl From<SimpleType> for DataType {
    fn from(s: SimpleType) -> Self {
        DataType::SimpleType(s)
    }
}
impl From<AggregateType> for DataType {
    fn from(s: AggregateType) -> Self {
        DataType::AggregateType(s)
    }
}

impl From<SimpleType> for Frame {
    fn from(st: SimpleType) -> Self {
        match st {
            SimpleType::Blob(bytes) => Frame::Bulk(bytes.into()),
            SimpleType::SimpleString(s) => Frame::Simple(s),
            SimpleType::Integer(n) => Frame::Integer(n),
            SimpleType::Null => Frame::Null,
        }
    }
}

impl From<SimpleType> for Bytes {
    fn from(st: SimpleType) -> Self {
        match st {
            SimpleType::Blob(bytes) => bytes.into(),
            SimpleType::SimpleString(s) => Bytes::from(s.into_bytes()),
            SimpleType::Integer(n) => Bytes::from(n.to_string().into_bytes()),
            SimpleType::Null => Bytes::new(),
        }
    }
}

impl From<&str> for SimpleType {
    fn from(s: &str) -> Self {
        SimpleType::SimpleString(s.to_owned())
    }
}
