use std::ops::Bound;

use dict::data_type::DataType;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SimpleTypePair {
    pub key: String,
    pub value: DataType,
}

#[derive(Debug)]
pub enum ZrangeItem {
    Rank((i64, i64)),
    Socre((Bound<f64>, Bound<f64>)),
    Lex((Bound<String>, Bound<String>)),
}
