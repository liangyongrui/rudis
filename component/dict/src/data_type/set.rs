use std::{
    collections::HashSet,
    ops::{Deref, DerefMut},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Set {
    pub inner: HashSet<Box<[u8]>, ahash::RandomState>,
}

impl Set {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Deref for Set {
    type Target = HashSet<Box<[u8]>, ahash::RandomState>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Set {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
