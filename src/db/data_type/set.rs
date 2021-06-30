use std::{ops::Deref, sync::Arc};

use rpds::HashTrieSetSync;

use super::{AggregateType, DataType, SimpleType};
use crate::db::{
    result::Result,
    slot::{Entry, Slot},
};

#[derive(Debug, Clone)]
pub struct Set {
    version: u64,
    value: Arc<HashTrieSetSync<SimpleType>>,
}
impl Deref for Set {
    type Target = HashTrieSetSync<SimpleType>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl Set {
    fn new_data_type() -> DataType {
        DataType::AggregateType(AggregateType::Set(Set::new()))
    }

    fn new() -> Self {
        Self {
            version: 0,
            value: Arc::new(HashTrieSetSync::new_sync()),
        }
    }
    fn process<T, F: FnOnce(&Set) -> T>(
        slot: &Slot,
        key: &str,
        f: F,
        none_value: fn() -> T,
    ) -> Result<T> {
        let entry = slot.entries.get(key);
        match entry {
            Some(e) => match e.value() {
                Entry {
                    data: DataType::AggregateType(AggregateType::Set(set)),
                    ..
                } => Ok(f(set)),
                _ => Err("the value stored at key is not a set.".to_owned()),
            },
            None => Ok(none_value()),
        }
    }

    fn mut_process<T, F: FnOnce(&mut Set) -> T>(
        slot: &Slot,
        key: &str,
        f: F,
        none_value: fn() -> T,
    ) -> Result<T> {
        let entry = slot.entries.get_mut(key);
        match entry {
            Some(mut e) => match e.value_mut() {
                Entry {
                    data: DataType::AggregateType(AggregateType::Set(set)),
                    ..
                } => Ok(f(set)),
                _ => Err("the value stored at key is not a set.".to_owned()),
            },
            None => Ok(none_value()),
        }
    }

    fn mut_process_exists_or_new<T, F: FnOnce(&mut Set) -> Result<T>>(
        slot: &Slot,
        key: &str,
        f: F,
    ) -> Result<T> {
        let mut entry = slot.get_or_insert_entry(&key, || (Set::new_data_type(), None));
        match entry.value_mut() {
            Entry {
                data: DataType::AggregateType(AggregateType::Set(set)),
                ..
            } => Ok(f(set)?),
            _ => Err("the value stored at key is not a set.".to_owned()),
        }
    }
}
impl Slot {
    pub fn sadd(&self, key: String, values: Vec<SimpleType>) -> Result<usize> {
        Set::mut_process_exists_or_new(self, &key, |set| {
            let old_len = set.size();
            let mut new: Option<HashTrieSetSync<SimpleType>> = None;
            for v in values {
                if let Some(ref mut n) = new {
                    n.insert_mut(v)
                } else {
                    new = Some(set.insert(v))
                }
            }
            if let Some(n) = new {
                set.version += 1;
                set.value = Arc::new(n);
            }
            Ok(set.size() - old_len)
        })
    }

    pub fn smismember(&self, key: &str, values: Vec<&SimpleType>) -> Result<Vec<bool>> {
        let set = Set::process(
            self,
            key,
            |set| Arc::clone(&set.value),
            || Arc::new(HashTrieSetSync::new_sync()),
        )?;
        Ok(values.into_iter().map(|t| set.contains(t)).collect())
    }

    pub fn smembers(&self, key: &str) -> Result<Arc<HashTrieSetSync<SimpleType>>> {
        Set::process(
            self,
            key,
            |set| Arc::clone(&set.value),
            || Arc::new(HashTrieSetSync::new_sync()),
        )
    }

    pub fn srem(&self, key: &str, values: Vec<&SimpleType>) -> Result<usize> {
        Set::mut_process(
            self,
            key,
            |set| {
                let old_len = set.size();
                let mut new: Option<HashTrieSetSync<SimpleType>> = None;
                for v in values {
                    if let Some(ref mut n) = new {
                        n.remove_mut(v);
                    } else {
                        new = Some(set.remove(v))
                    }
                }
                if let Some(n) = new {
                    set.version += 1;
                    set.value = Arc::new(n);
                }
                set.size() - old_len
            },
            || 0,
        )
    }
}
