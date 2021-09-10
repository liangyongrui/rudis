use std::collections::{BTreeSet, HashMap};

use serde::{Deserialize, Serialize};

use super::Float;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct Node {
    pub score: Float,
    pub key: Box<[u8]>,
}
impl From<(Float, Box<[u8]>)> for Node {
    fn from(o: (Float, Box<[u8]>)) -> Self {
        Self {
            score: o.0,
            key: o.1,
        }
    }
}
impl Node {
    pub fn new(key: Box<[u8]>, score: f64) -> Self {
        Self {
            key,
            score: Float(score),
        }
    }
}
/// key value pairs
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct SortedSet {
    pub hash: HashMap<Box<[u8]>, Node, ahash::RandomState>,
    pub value: BTreeSet<Node>,
}

impl SortedSet {
    pub fn new() -> Self {
        Self::default()
    }
}
