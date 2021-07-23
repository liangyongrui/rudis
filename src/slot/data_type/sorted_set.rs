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
