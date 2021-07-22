use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use chrono::{DateTime, Utc};

use super::data_type::{DataType, SimpleType};

pub struct Dict {
    pub inner: HashMap<SimpleType, Value>,
}

pub struct Value {
    pub id: u64,
    pub data: DataType,
    pub expire_at: Option<DateTime<Utc>>,
}

impl Dict {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }
}

impl Deref for Dict {
    type Target = HashMap<SimpleType, Value>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Dict {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
