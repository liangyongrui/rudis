mod blob;
mod hash;
mod list;
mod number;
use std::convert::TryFrom;

use bytes::Bytes;
pub use hash::HashEntry;

pub use self::blob::Blob;
use self::{hash::Hash, list::List, number::Number};
use crate::Frame;

#[derive(Debug, Clone)]
pub enum DataType {
    SimpleType(SimpleType),
    AggregateType(AggregateType),
}
#[derive(Debug, Clone)]
pub enum SimpleType {
    Blob(Blob),
    SimpleString(String),
    Number(Number),
    // Bool(bool),
    // todo
    VerbatimString,
    // todo
    BigNumber,
}

#[derive(Debug, Clone)]
pub enum AggregateType {
    List(List),
    Hash(Hash),
    Set,
    SortedSet,
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
            SimpleType::Blob(bytes) => Frame::Bulk(bytes.get_inner()),
            SimpleType::SimpleString(s) => Frame::Simple(s),
            SimpleType::Number(n) => Frame::Integer(n.0),
            SimpleType::VerbatimString => todo!(),
            SimpleType::BigNumber => todo!(),
        }
    }
}

impl From<SimpleType> for Bytes {
    fn from(st: SimpleType) -> Self {
        match st {
            SimpleType::Blob(bytes) => bytes.get_inner(),
            SimpleType::SimpleString(s) => Bytes::from(s.into_bytes()),
            SimpleType::Number(n) => Bytes::from(n.to_string().into_bytes()),
            SimpleType::VerbatimString => todo!(),
            SimpleType::BigNumber => todo!(),
        }
    }
}
