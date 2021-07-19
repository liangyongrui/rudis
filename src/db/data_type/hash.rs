use std::ops::Deref;

use rpds::HashTrieMapSync;
use serde::{Deserialize, Serialize};

use super::{AggregateType, DataType, SimpleType, SimpleTypePair};
use crate::db::{
    result::Result,
    slot::{Entry, Slot},
};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hash {
    version: u64,
    value: HashTrieMapSync<SimpleType, SimpleType>,
}

impl Deref for Hash {
    type Target = HashTrieMapSync<SimpleType, SimpleType>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl Hash {
    fn new_data_type() -> DataType {
        DataType::AggregateType(AggregateType::Hash(Hash::new()))
    }

    fn new() -> Self {
        Self {
            version: 0,
            value: HashTrieMapSync::new_sync(),
        }
    }

    fn process<T, F: FnOnce(&Hash) -> T>(
        slot: &Slot,
        key: &SimpleType,
        f: F,
        none_value: fn() -> T,
    ) -> Result<T> {
        let entry = slot.entries.get(key);
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

    fn mut_process<T, F: FnOnce(&mut Hash) -> T>(
        slot: &Slot,
        key: &SimpleType,
        f: F,
        none_value: fn() -> T,
    ) -> Result<T> {
        let entry = slot.entries.get_mut(key);
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

    fn mut_process_exists_or_new<T, F: FnOnce(&mut Hash) -> Result<T>>(
        slot: &Slot,
        key: SimpleType,
        f: F,
    ) -> Result<T> {
        let mut entry = slot.get_or_insert_entry(key, || (Hash::new_data_type(), None));
        match entry.value_mut() {
            Entry {
                data: DataType::AggregateType(AggregateType::Hash(hash)),
                ..
            } => Ok(f(hash)?),
            _ => Err("the value stored at key is not a hash.".to_owned()),
        }
    }
}
impl Slot {
    pub fn hset(&self, key: SimpleType, pairs: Vec<SimpleTypePair>) -> Result<usize> {
        Hash::mut_process_exists_or_new(self, key, |hash| {
            let len = pairs.len();
            let mut new = hash.value.clone();
            for SimpleTypePair { key: field, value } in pairs.into_iter() {
                new.insert_mut(field, value);
            }
            hash.version += 1;
            hash.value = new;
            Ok(len)
        })
    }

    pub fn hsetnx(&self, key: SimpleType, field: SimpleType, value: SimpleType) -> Result<usize> {
        Hash::mut_process_exists_or_new(self, key, |hash| {
            if hash.contains_key(&field) {
                Ok(0)
            } else {
                hash.version += 1;
                hash.value = hash.insert(field, value);
                Ok(1)
            }
        })
    }

    pub fn hgetall(&self, key: &SimpleType) -> Result<Vec<SimpleTypePair>> {
        Hash::process(
            self,
            key,
            |hash| hash.value.clone(),
            HashTrieMapSync::new_sync,
        )
        .map(|p| {
            p.iter()
                .map(|x| SimpleTypePair {
                    key: x.0.clone(),
                    value: x.1.clone(),
                })
                .collect()
        })
    }

    pub fn hmget(
        &self,
        key: &SimpleType,
        fields: Vec<SimpleType>,
    ) -> Result<Vec<Option<SimpleType>>> {
        Hash::process(
            self,
            key,
            |hash| hash.value.clone(),
            HashTrieMapSync::new_sync,
        )
        .map(|p| fields.into_iter().map(|x| p.get(&x).cloned()).collect())
    }

    pub fn hdel(&self, key: &SimpleType, fields: Vec<SimpleType>) -> Result<usize> {
        Hash::mut_process(
            self,
            key,
            |hash| {
                let old_len = hash.size();
                let mut new = hash.value.clone();
                for field in fields {
                    new.remove_mut(&field);
                }
                hash.version += 1;
                hash.value = new;
                old_len - hash.size()
            },
            || 0,
        )
    }

    pub fn hexists(&self, key: &SimpleType, field: SimpleType) -> Result<usize> {
        Hash::process(
            self,
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

    pub fn hincrby(&self, key: SimpleType, field: SimpleType, value: i64) -> Result<i64> {
        Hash::mut_process_exists_or_new(self, key, |hash| {
            let old_value = match hash.get(&field) {
                Some(SimpleType::SimpleString(s)) => s.parse::<i64>().map_err(|e| e.to_string())?,
                Some(SimpleType::Integer(i)) => *i,
                Some(_) => return Err("type not support".to_owned()),
                None => 0,
            };
            let nv = old_value + value;
            hash.version += 1;
            hash.value = hash.insert(field, SimpleType::Integer(nv));
            Ok(nv)
        })
    }
}

#[cfg(test)]
mod test {
    use crate::{
        db::{data_type::SimpleTypePair, slot::Slot},
        SimpleType,
    };

    #[tokio::test]
    async fn test() {
        let slot = Slot::new();
        let key = SimpleType::SimpleString("abc".to_string());
        assert_eq!(
            Ok(2),
            slot.hset(
                key.clone(),
                vec![
                    SimpleTypePair {
                        key: "abc".into(),
                        value: "456".into(),
                    },
                    SimpleTypePair {
                        key: "def".into(),
                        value: 123.into(),
                    },
                ],
            )
        );
        assert_eq!(
            slot.hmget(&key, vec!["abc".into(), "aaa".into(), "def".into()]),
            Ok(vec![Some("456".into()), None, Some(123.into())])
        );
        assert_eq!(slot.hsetnx(key.clone(), "abc".into(), "111".into()), Ok(0));
        assert_eq!(slot.hsetnx(key.clone(), "aaa".into(), "111".into()), Ok(1));
        assert_eq!(
            slot.hmget(&key, vec!["abc".into(), "aaa".into()]),
            Ok(vec![Some("456".into()), Some("111".into())])
        );
        assert_eq!(
            {
                let mut r = slot.hgetall(&key).unwrap();
                r.sort();
                r
            },
            {
                let mut r2 = vec![
                    SimpleTypePair {
                        key: "abc".into(),
                        value: "456".into(),
                    },
                    SimpleTypePair {
                        key: "aaa".into(),
                        value: "111".into(),
                    },
                    SimpleTypePair {
                        key: "def".into(),
                        value: 123.into(),
                    },
                ];
                r2.sort();
                r2
            }
        );
        assert_eq!(
            slot.hdel(&key, vec!["abc".into(), "aaa".into(), "xxx".into()]),
            Ok(2)
        );
        assert_eq!(
            slot.hgetall(&key).unwrap(),
            vec![SimpleTypePair {
                key: "def".into(),
                value: 123.into(),
            }]
        );
        assert_eq!(slot.hexists(&key, "abc".into()), Ok(0));
        assert_eq!(slot.hexists(&key, "def".into()), Ok(1));
        assert_eq!(slot.hincrby(key.clone(), "def".into(), 123), Ok(123 + 123));
        assert_eq!(slot.hincrby(key.clone(), "xxx".into(), 123), Ok(123));
        slot.hset(
            key.clone(),
            vec![SimpleTypePair {
                key: "abc".into(),
                value: "456".into(),
            }],
        )
        .unwrap();
        assert_eq!(slot.hincrby(key, "abc".into(), 123), Ok(456 + 123));
    }
}
