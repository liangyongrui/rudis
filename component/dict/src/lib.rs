#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::doc_markdown)]
#![allow(unstable_name_collisions)]
#![allow(clippy::semicolon_if_nothing_returned)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::enum_glob_use)]
#![allow(clippy::missing_errors_doc)] //
#![allow(clippy::let_underscore_drop)] //
#![allow(clippy::missing_panics_doc)] //
#![allow(clippy::single_match_else)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::shadow_unrelated)]

pub mod cmd;
pub mod data_type;

use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use common::now_timestamp_ms;
use data_type::DataType;
use serde::{Deserialize, Serialize};
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Dict {
    pub write_id: u64,
    pub inner: HashMap<Arc<[u8]>, Value, ahash::RandomState>,
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Value {
    pub data: DataType,
    /// unix timestamp ms
    /// 0 表示不过期
    pub expires_at: u64,
}

impl Dict {
    #[inline]
    pub fn next_id(&mut self) -> u64 {
        self.write_id += 1;
        self.write_id
    }

    pub fn last_write_op_id(&self) -> u64 {
        self.write_id
    }

    #[inline]
    pub fn d_exists(&self, key: &[u8]) -> bool {
        self.d_get(key).is_some()
    }

    #[inline]
    pub fn d_get(&self, key: &[u8]) -> Option<&Value> {
        self.get(key)
            .filter(|v| v.expires_at == 0 || v.expires_at > now_timestamp_ms())
    }

    #[inline]
    pub fn d_get_mut(&mut self, key: &[u8]) -> Option<&mut Value> {
        self.get_mut(key)
            .filter(|v| v.expires_at == 0 || v.expires_at > now_timestamp_ms())
    }

    pub fn d_get_mut_or_insert_with<F: FnOnce() -> Value>(
        &mut self,
        key: Arc<[u8]>,
        f: F,
    ) -> &mut Value {
        match self.entry(key) {
            std::collections::hash_map::Entry::Occupied(mut o) => {
                let expires_at = o.get().expires_at;
                if expires_at > 0 && expires_at <= now_timestamp_ms() {
                    *o.get_mut() = f();
                }
                o.into_mut()
            }
            std::collections::hash_map::Entry::Vacant(e) => e.insert(f()),
        }
    }
}

impl Deref for Dict {
    type Target = HashMap<Arc<[u8]>, Value, ahash::RandomState>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Dict {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
