//! slot 每个value的数据类型
//!
//! 类型主要分为两种，简单类型 和 集合类型

use std::{convert::TryFrom, sync::Arc};
mod deque;
mod kvp;
mod set;
pub mod sorted_set;

use serde::{Deserialize, Serialize};

pub use self::{deque::Deque, kvp::Kvp, set::Set, sorted_set::SortedSet};
pub use crate::utils::float::Float;

/// slot value 的类型
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum DataType {
    SimpleType(SimpleType),
    CollectionType(CollectionType),
}

/// 简单类型
///
/// When derived on enums, variants are ordered by their top-to-bottom discriminant order.
#[derive(PartialOrd, Ord, PartialEq, Eq, Debug, Hash, Clone, Deserialize, Serialize)]
pub enum SimpleType {
    /// 占位，用于排序
    Big,
    String(Arc<str>),
    Bytes(Arc<[u8]>),
    Integer(i64),
    Float(Float),
    Null,
}

impl From<&str> for SimpleType {
    fn from(s: &str) -> Self {
        SimpleType::String(s.into())
    }
}
impl From<String> for SimpleType {
    fn from(s: String) -> Self {
        SimpleType::String(s.into())
    }
}
impl From<i64> for SimpleType {
    fn from(s: i64) -> Self {
        SimpleType::Integer(s)
    }
}

impl TryFrom<&SimpleType> for i64 {
    type Error = crate::Error;

    fn try_from(value: &SimpleType) -> Result<Self, Self::Error> {
        let res = match value {
            SimpleType::String(s) => s.parse()?,
            SimpleType::Bytes(b) => std::str::from_utf8(b)?.parse()?,
            SimpleType::Integer(i) => *i,
            SimpleType::Float(_) => return Err("type error".into()),
            SimpleType::Null => 0,
            SimpleType::Big => 0,
        };
        Ok(res)
    }
}

impl TryFrom<&SimpleType> for f64 {
    type Error = crate::Error;

    fn try_from(value: &SimpleType) -> Result<Self, Self::Error> {
        let res = match value {
            SimpleType::String(s) => s.to_string().parse()?,
            SimpleType::Bytes(b) => std::str::from_utf8(b)?.parse()?,
            SimpleType::Integer(i) => *i as _,
            SimpleType::Float(f) => f.0,
            SimpleType::Null => 0f64,
            SimpleType::Big => 0f64,
        };
        Ok(res)
    }
}

impl From<i64> for DataType {
    fn from(i: i64) -> Self {
        DataType::SimpleType(SimpleType::Integer(i))
    }
}

/// 集合类型
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum CollectionType {
    Kvp(Kvp),
    Deque(Deque),
    Set(Set),
    SortedSet(SortedSet),
}
