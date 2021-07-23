use std::collections::HashMap;

use rpds::RedBlackTreeSetSync;
use serde::{Deserialize, Serialize};

use super::Float;
use crate::slot::data_type::SimpleType;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct Node {
    pub key: SimpleType,
    pub score: Float,
}

/// key value pairs
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SortedSet {
    pub hash: HashMap<SimpleType, Node>,
    pub value: RedBlackTreeSetSync<Node>,
}

impl SortedSet {
    pub fn new() -> Self {
        Self::default()
    }
}

pub fn sharp_limit(limit: Option<(i64, i64)>, len: usize) -> (usize, usize) {
    let (offset, count) = match limit {
        Some((mut offset, count)) => {
            if offset < 0 {
                offset = 0;
            }
            (
                offset as usize,
                if count < 0 {
                    len
                } else {
                    (offset + count) as usize
                },
            )
        }
        None => (0, len),
    };
    (offset, count)
}