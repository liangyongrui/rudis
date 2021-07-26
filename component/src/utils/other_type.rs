use std::{net::SocketAddr, ops::Bound};

use crate::slot::data_type::SimpleType;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SimpleTypePair {
    pub key: SimpleType,
    pub value: SimpleType,
}

#[derive(Debug)]
pub enum Role {
    Master(Vec<SocketAddr>),
    Replica(Option<SocketAddr>),
}

#[derive(Debug)]
pub enum ZrangeItem {
    Rank((i64, i64)),
    Socre((Bound<f64>, Bound<f64>)),
    Lex((Bound<String>, Bound<String>)),
}