use std::hash;

use serde::{Deserialize, Serialize};

/// Custom f64, in order to realize Ord, Eq.
///
/// # Warnings
///
/// NaN is not allowed.
#[derive(Debug, PartialEq, Clone, Copy, Deserialize, Serialize)]
pub struct Float(pub f64);
impl Eq for Float {}

impl PartialOrd for Float {
    #[inline]
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
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Float
        self.partial_cmp(other).unwrap()
    }
}
#[allow(clippy::derive_hash_xor_eq)]
impl hash::Hash for Float {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.0.to_be_bytes().hash(state);
    }
}
