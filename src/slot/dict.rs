use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use chrono::{DateTime, Utc};

use super::data_type::{DataType, SimpleType};
use crate::db;

pub struct Dict {
    pub inner: HashMap<SimpleType, Value>,
}
#[derive(Debug)]
pub struct Value {
    pub id: u64,
    pub data: DataType,
    pub expire_at: Option<DateTime<Utc>>,
}

impl Dict {
    #[inline]
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    #[inline]
    pub fn d_exists(&self, key: &SimpleType) -> bool {
        self.d_get(key).is_some()
    }

    #[inline]
    pub fn d_get(&self, key: &SimpleType) -> Option<&Value> {
        self.get(key)
            .filter(|v| v.expire_at.filter(|x| *x <= Utc::now()).is_none())
    }

    #[inline]
    pub fn d_get_mut(&mut self, key: &SimpleType) -> Option<&mut Value> {
        self.get_mut(key)
            .filter(|v| v.expire_at.filter(|x| *x <= Utc::now()).is_none())
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
