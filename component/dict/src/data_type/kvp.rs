use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use serde::{Deserialize, Serialize};

use crate::data_type::DataType;
/// key value pairs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Kvp {
    pub inner: HashMap<Box<[u8]>, DataType, ahash::RandomState>,
}

impl Kvp {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Deref for Kvp {
    type Target = HashMap<Box<[u8]>, DataType, ahash::RandomState>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Kvp {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
