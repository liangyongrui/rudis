use std::ops::Deref;

use rpds::HashTrieSetSync;
use serde::{Deserialize, Serialize};

use super::{AggregateType, DataType, SimpleType};
use crate::db::{dict, result::Result, slot::Slot};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Set {
    version: u64,
    value: HashTrieSetSync<SimpleType>,
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
            value: HashTrieSetSync::new_sync(),
        }
    }
    fn process<T, F: FnOnce(&Set) -> T, NF: FnOnce() -> T>(
        slot: &Slot,
        key: &SimpleType,
        f: F,
        none_value: NF,
    ) -> Result<T> {
        let entry = slot.dict.get(key);
        match entry {
            Some(e) => match e {
                dict::Entry {
                    data: DataType::AggregateType(AggregateType::Set(ref set)),
                    ..
                } => Ok(f(set)),
                _ => Err("the value stored at key is not a set.".to_owned()),
            },
            None => Ok(none_value()),
        }
    }

    fn mut_process<T, F: FnOnce(&mut Set) -> T>(
        slot: &Slot,
        key: &SimpleType,
        f: F,
        none_value: fn() -> T,
    ) -> Result<T> {
        slot.dict.process_mut(key, |entry| match entry {
            Some(e) => match e {
                dict::Entry {
                    data: DataType::AggregateType(AggregateType::Set(set)),
                    ..
                } => Ok(f(set)),
                _ => Err("the value stored at key is not a set.".to_owned()),
            },
            None => Ok(none_value()),
        })
    }

    fn mut_process_exists_or_new<T, F: FnOnce(&mut Set) -> Result<T>>(
        slot: &Slot,
        key: SimpleType,
        f: F,
    ) -> Result<T> {
        slot.get_or_insert(
            key,
            || (Set::new_data_type(), None),
            |entry| match entry {
                dict::Entry {
                    data: DataType::AggregateType(AggregateType::Set(set)),
                    ..
                } => Ok(f(set)?),
                _ => Err("the value stored at key is not a set.".to_owned()),
            },
        )
    }
}
impl Slot {
    pub fn sadd(&self, key: SimpleType, values: Vec<SimpleType>) -> Result<usize> {
        Set::mut_process_exists_or_new(self, key, |set| {
            let old_len = set.size();
            let mut new = set.value.clone();
            for v in values {
                new.insert_mut(v)
            }
            set.version += 1;
            set.value = new;
            Ok(set.size() - old_len)
        })
    }

    pub fn smismember(&self, key: &SimpleType, values: Vec<SimpleType>) -> Result<Vec<bool>> {
        let len = values.len();
        let n = || vec![false; len];
        let set = Set::process(
            self,
            key,
            |set| values.into_iter().map(|t| set.contains(&t)).collect(),
            n,
        )?;
        Ok(set)
    }

    pub fn smembers(&self, key: &SimpleType) -> Result<HashTrieSetSync<SimpleType>> {
        Set::process(
            self,
            key,
            |set| set.value.clone(),
            HashTrieSetSync::new_sync,
        )
    }

    pub fn srem(&self, key: &SimpleType, values: Vec<SimpleType>) -> Result<usize> {
        Set::mut_process(
            self,
            key,
            |set| {
                let old_len = set.size();
                let mut new = set.value.clone();
                for v in values {
                    new.remove_mut(&v);
                }
                set.version += 1;
                set.value = new;
                old_len - set.size()
            },
            || 0,
        )
    }
}

#[cfg(test)]
mod test {
    use crate::db::slot::Slot;

    #[tokio::test]
    async fn test() {
        let slot = Slot::new();
        assert_eq!(
            slot.sadd(
                "key".into(),
                vec!["123".into(), 2.into(), 3.into(), 4.into(), 2.into()],
            ),
            Ok(4)
        );
        assert_eq!(
            slot.smismember(&"key".into(), vec!["123".into(), 5.into(), 4.into()]),
            Ok(vec![true, false, true])
        );
        assert_eq!(
            slot.srem(&"key".into(), vec!["123".into(), 5.into(), 4.into()]),
            Ok(2)
        );
        assert_eq!(slot.sadd("key".into(), vec!["bb".into()]), Ok(1));
        let r = slot.smembers(&"key".into()).unwrap();
        let mut r2 = r.iter().cloned().collect::<Vec<_>>();
        r2.sort();
        assert_eq!(r2, vec![2.into(), 3.into(), "bb".into(),])
    }
}
