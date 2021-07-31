use std::ops::Bound;

use crate::slot::data_type::SimpleType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleTypePair {
    pub key: String,
    pub value: SimpleType,
}

#[derive(Debug)]
pub enum ZrangeItem {
    Rank((i64, i64)),
    Socre((Bound<f64>, Bound<f64>)),
    Lex((Bound<String>, Bound<String>)),
}
