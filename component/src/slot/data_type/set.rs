use std::ops::{Deref, DerefMut};

use rpds::HashTrieSetSync;
use serde::{Deserialize, Serialize};

use crate::slot::data_type::SimpleType;

#[derive(Debug, Serialize, Deserialize,PartialEq, Eq)]
pub struct Set {
    inner: HashTrieSetSync<SimpleType>,
}

impl Set {
    pub fn new() -> Self {
        Self {
            inner: HashTrieSetSync::new_sync(),
        }
    }
}

impl Default for Set {
    fn default() -> Self {
        Self::new()
    }
}
impl Deref for Set {
    type Target = HashTrieSetSync<SimpleType>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Set {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
