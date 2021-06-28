use std::{ops::Deref, sync::Arc, usize};

use rpds::HashTrieMapSync;

use super::{number::Number, AggregateType, DataType, SimpleType};
use crate::db::{
    result::Result,
    slot::{Entry, Slot},
};
#[derive(Debug, Clone)]
pub struct Hash {
    version: u64,
    value: Arc<HashTrieMapSync<String, SimpleType>>,
}
impl Deref for Hash {
    type Target = HashTrieMapSync<String, SimpleType>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
#[derive(Debug)]
pub struct HashEntry {
    pub field: String,
    pub value: SimpleType,
}

impl Hash {
    fn new_data_type() -> DataType {
        DataType::AggregateType(AggregateType::Hash(Hash::new()))
    }

    fn new() -> Self {
        Self {
            version: 0,
            value: Arc::new(HashTrieMapSync::new_sync()),
        }
    }
}
impl Slot {
    fn process_hash<T, F: FnOnce(&Hash) -> T>(
        &self,
        key: &str,
        f: F,
        none_value: fn() -> T,
    ) -> Result<T> {
        let entry = self.entries.get(key);
        match entry {
            Some(e) => match e.value() {
                Entry {
                    data: DataType::AggregateType(AggregateType::Hash(hash)),
                    ..
                } => Ok(f(hash)),
                _ => Err("the value stored at key is not a hash.".to_owned()),
            },
            None => Ok(none_value()),
        }
    }

    fn mut_process_hash<T, F: FnOnce(&mut Hash) -> T>(
        &self,
        key: &str,
        f: F,
        none_value: fn() -> T,
    ) -> Result<T> {
        let entry = self.entries.get_mut(key);
        match entry {
            Some(mut e) => match e.value_mut() {
                Entry {
                    data: DataType::AggregateType(AggregateType::Hash(hash)),
                    ..
                } => Ok(f(hash)),
                _ => Err("the value stored at key is not a hash.".to_owned()),
            },
            None => Ok(none_value()),
        }
    }

    fn mut_process_exists_or_new_hash<T, F: FnOnce(&mut Hash) -> Result<T>>(
        &self,
        key: &str,
        f: F,
    ) -> Result<T> {
        let mut entry = self.get_or_insert_entry(&key, || (Hash::new_data_type(), None));
        match entry.value_mut() {
            Entry {
                data: DataType::AggregateType(AggregateType::Hash(hash)),
                ..
            } => Ok(f(hash)?),
            _ => Err("the value stored at key is not a hash.".to_owned()),
        }
    }

    pub fn hset(&self, key: String, pairs: Vec<HashEntry>) -> Result<usize> {
        self.mut_process_exists_or_new_hash(&key, |hash| {
            let len = pairs.len();
            let mut new: Option<HashTrieMapSync<String, SimpleType>> = None;
            for HashEntry { field, value } in pairs.into_iter() {
                if let Some(ref mut n) = new {
                    n.insert_mut(field, value);
                } else {
                    new = Some(hash.insert(field, value));
                }
            }
            if let Some(n) = new {
                hash.version += 1;
                hash.value = Arc::new(n);
            }
            Ok(len)
        })
    }

    pub fn hsetnx(&self, key: &str, field: String, value: SimpleType) -> Result<usize> {
        self.mut_process_exists_or_new_hash(&key, |hash| {
            if hash.contains_key(&field) {
                Ok(0)
            } else {
                hash.version += 1;
                hash.value = Arc::new(hash.insert(field, value));
                Ok(1)
            }
        })
    }

    pub fn hgetall(&self, key: &str) -> Result<Vec<HashEntry>> {
        self.process_hash(
            key,
            |hash| Arc::clone(&hash.value),
            || Arc::new(HashTrieMapSync::new_sync()),
        )
        .map(|p| {
            p.iter()
                .map(|x| HashEntry {
                    field: x.0.clone(),
                    value: x.1.clone(),
                })
                .collect()
        })
    }

    pub fn hmget(&self, key: &str, fields: Vec<String>) -> Result<Vec<Option<SimpleType>>> {
        self.process_hash(
            key,
            |hash| Arc::clone(&hash.value),
            || Arc::new(HashTrieMapSync::new_sync()),
        )
        .map(|p| fields.into_iter().map(|x| p.get(&x).cloned()).collect())
    }

    pub fn hdel(&self, key: &str, fields: Vec<String>) -> Result<usize> {
        self.mut_process_hash(
            key,
            |hash| {
                let mut count = 0;
                let mut new: Option<HashTrieMapSync<String, SimpleType>> = None;
                for field in fields {
                    if let Some(ref mut n) = new {
                        if n.remove_mut(&field) {
                            count += 1
                        }
                    } else {
                        if hash.contains_key(&field) {
                            count += 1;
                        }
                        new = Some(hash.remove(&field));
                    }
                }
                if let Some(n) = new {
                    hash.version += 1;
                    hash.value = Arc::new(n);
                }
                count
            },
            || 0,
        )
    }

    pub fn hexists(&self, key: &str, field: String) -> Result<usize> {
        self.process_hash(
            key,
            |hash| {
                if hash.value.contains_key(&field) {
                    1
                } else {
                    0
                }
            },
            || 0,
        )
    }

    pub fn hincrby(&self, key: &str, field: String, value: i64) -> Result<i64> {
        self.mut_process_exists_or_new_hash(&key, |hash| {
            let old_value = match hash.get(&field) {
                Some(SimpleType::SimpleString(s)) => s.parse::<i64>().map_err(|e| e.to_string())?,
                Some(SimpleType::Number(i)) => (i.0),
                Some(_) => return Err("type not support".to_owned()),
                None => 0,
            };
            let nv = old_value + value;
            hash.version += 1;
            hash.value = Arc::new(hash.insert(field, SimpleType::Number(Number(nv))));
            Ok(nv)
        })
    }
}
