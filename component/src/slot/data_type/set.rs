use std::ops::{Deref, DerefMut};

use rpds::HashTrieSetSync;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Set {
    pub inner: Box<HashTrieSetSync<String>>,
}

impl Set {
    pub fn new() -> Self {
        Self {
            inner: Box::new(HashTrieSetSync::new_sync()),
        }
    }
}

impl Default for Set {
    fn default() -> Self {
        Self::new()
    }
}
impl Deref for Set {
    type Target = HashTrieSetSync<String>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Set {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
