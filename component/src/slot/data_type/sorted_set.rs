use std::collections::{BTreeSet, HashMap};

use serde::{Deserialize, Serialize};

use super::Float;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct Node {
    pub score: Float,
    pub key: String,
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
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct SortedSet {
    pub hash: Box<HashMap<String, Node, ahash::RandomState>>,
    pub value: Box<BTreeSet<Node>>,
}

impl SortedSet {
    pub fn new() -> Self {
        Self::default()
    }
}
