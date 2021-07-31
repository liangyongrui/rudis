use std::collections::HashMap;

use rpds::RedBlackTreeSetSync;
use serde::{Deserialize, Serialize};

use super::Float;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct Node {
    pub key: String,
    pub score: Float,
}

impl Node {
    pub fn new(key: String, score: f64) -> Self {
        Self {
            key,
            score: Float(score),
        }
    }
}
/// key value pairs
#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct SortedSet {
    pub hash: HashMap<String, Node>,
    pub value: RedBlackTreeSetSync<Node>,
}

impl SortedSet {
    pub fn new() -> Self {
        Self::default()
    }
}
