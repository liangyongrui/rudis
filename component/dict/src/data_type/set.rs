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
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Deref for Set {
    type Target = HashSet<Box<[u8]>, ahash::RandomState>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Set {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
