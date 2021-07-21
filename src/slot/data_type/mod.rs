//! slot 每个value的数据类型
//!
//! 类型主要分为两种，简单类型 和 集合类型

use bytes::Bytes;
use serde::{Deserialize, Serialize};

use crate::utils::float::Float;

/// slot value 的类型
#[derive(Debug)]
pub enum DataType {
    SimpleType(SimpleType),
    CollectionType(CollectionType),
}

/// 简单类型
///
/// When derived on enums, variants are ordered by their top-to-bottom discriminant order.
#[derive(PartialOrd, Ord, PartialEq, Eq, Debug, Hash, Clone, Deserialize, Serialize)]
pub enum SimpleType {
    String(String),
    Bytes(Vec<u8>),
    Integer(i64),
    Float(Float),
    Null,
}

/// 集合类型
/// todo
#[derive(Debug)]
pub enum CollectionType {}
