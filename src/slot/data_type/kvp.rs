use std::ops::Deref;

use rpds::HashTrieMapSync;
use serde::{Deserialize, Serialize};

use crate::slot::data_type::SimpleType;
/// key value pairs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kvp {
    inner: HashTrieMapSync<SimpleType, SimpleType>,
}

impl Kvp {
    pub fn new() -> Self {
        Self {
            inner: HashTrieMapSync::new_sync(),
        }
    }
}
impl Deref for Kvp {
    type Target = HashTrieMapSync<SimpleType, SimpleType>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
