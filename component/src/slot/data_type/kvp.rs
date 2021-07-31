use std::ops::{Deref, DerefMut};

use rpds::HashTrieMapSync;
use serde::{Deserialize, Serialize};

use crate::slot::data_type::DataType;
/// key value pairs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Kvp {
    pub inner: HashTrieMapSync<String, DataType>,
}

impl Kvp {
    pub fn new() -> Self {
        Self {
            inner: HashTrieMapSync::new_sync(),
        }
    }
}

impl Default for Kvp {
    fn default() -> Self {
        Self::new()
    }
}
impl Deref for Kvp {
    type Target = HashTrieMapSync<String, DataType>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Kvp {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
