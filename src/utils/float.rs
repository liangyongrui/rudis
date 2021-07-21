use std::hash;

use serde::{Deserialize, Serialize};

/// 自定义的f64，为了实现 Ord, Eq
#[derive(Debug, PartialEq, Clone, Copy, Deserialize, Serialize)]
pub struct Float(f64);
impl Eq for Float {}

impl PartialOrd for Float {
    #[allow(clippy::float_cmp)]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.0 > other.0 {
            Some(std::cmp::Ordering::Greater)
        } else if self.0 < other.0 {
            Some(std::cmp::Ordering::Less)
        } else if self.0 == other.0 {
            Some(std::cmp::Ordering::Equal)
        } else {
            None
        }
    }
}

impl Ord for Float {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}
#[allow(clippy::derive_hash_xor_eq)]
impl hash::Hash for Float {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.0.to_be_bytes().hash(state)
    }
}
