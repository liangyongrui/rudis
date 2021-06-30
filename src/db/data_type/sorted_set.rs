use std::{collections::HashMap, sync::Arc};

use rpds::RedBlackTreeSetSync;

use super::{AggregateType, DataType};
use crate::{
    db::{
        result::Result,
        slot::{Entry, Slot},
    },
    options::{GtLt, NxXx},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    score: f64,
    key: String,
}
impl Eq for Node {}
impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.score.partial_cmp(&other.score).map(|x| {
            if x.is_eq() {
                self.key.cmp(&other.key)
            } else {
                x
            }
        })
    }
}
impl Ord for Node {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(&other).expect("f64 can not Nan")
    }
}

#[derive(Debug, Clone)]
pub struct SortedSet {
    version: u64,
    hash: HashMap<String, Node>,
    value: Arc<RedBlackTreeSetSync<Node>>,
}

impl SortedSet {
    fn contains_key(&self, key: &str) -> bool {
        self.hash.contains_key(key)
    }
    fn get(&self, key: &str) -> Option<&Node> {
        self.hash.get(key)
    }
    fn can_update(&self, node: &Node, nx_xx: NxXx, gt_lt: GtLt) -> bool {
        match (nx_xx, gt_lt) {
            (NxXx::None, GtLt::None) => true,
            (NxXx::Nx, GtLt::None) => !self.contains_key(&node.key),
            (NxXx::Nx, _) => false,
            (NxXx::Xx, GtLt::None) => self.contains_key(&node.key),
            (_, GtLt::Gt) => self
                .get(&node.key)
                .filter(|n| node.score > n.score)
                .is_some(),
            (_, GtLt::Lt) => self
                .get(&node.key)
                .filter(|n| node.score < n.score)
                .is_some(),
        }
    }
    fn add(&mut self, values: Vec<Node>, nx_xx: NxXx, gt_lt: GtLt, ch: bool, incr: bool) -> usize {
        let old_len = self.value.size();
        let mut new = (*self.value).clone();
        for mut v in values {
            if self.can_update(&v, nx_xx, gt_lt) {
                if let Some(ov) = self.hash.remove(&v.key) {
                    new.remove_mut(&ov);
                    if incr {
                        v.score += ov.score;
                    }
                }
                self.hash.insert(v.key.clone(), v.clone());
                new.insert_mut(v)
            }
        }
        self.value.size() - if ch { 0 } else { old_len }
    }
}

impl SortedSet {
    fn new_data_type() -> DataType {
        DataType::AggregateType(AggregateType::SortedSet(SortedSet::new()))
    }

    fn new() -> Self {
        Self {
            version: 0,
            hash: HashMap::new(),
            value: Arc::new(RedBlackTreeSetSync::new_sync()),
        }
    }

    fn mut_process_exists_or_new<T, F: FnOnce(&mut SortedSet) -> Result<T>>(
        slot: &Slot,
        key: &str,
        f: F,
    ) -> Result<T> {
        let mut entry = slot.get_or_insert_entry(&key, || (SortedSet::new_data_type(), None));
        match entry.value_mut() {
            Entry {
                data: DataType::AggregateType(AggregateType::SortedSet(sorted_set)),
                ..
            } => Ok(f(sorted_set)?),
            _ => Err("the value stored at key is not a sorted set.".to_owned()),
        }
    }
    fn process<T, F: FnOnce(&SortedSet) -> T>(
        slot: &Slot,
        key: &str,
        f: F,
        none_value: fn() -> T,
    ) -> Result<T> {
        let entry = slot.entries.get(key);
        match entry {
            Some(e) => match e.value() {
                Entry {
                    data: DataType::AggregateType(AggregateType::SortedSet(sorted_set)),
                    ..
                } => Ok(f(sorted_set)),
                _ => Err("the value stored at key is not a sorted set.".to_owned()),
            },
            None => Ok(none_value()),
        }
    }

    fn mut_process<T, F: FnOnce(&mut SortedSet) -> T>(
        slot: &Slot,
        key: &str,
        f: F,
        none_value: fn() -> T,
    ) -> Result<T> {
        let entry = slot.entries.get_mut(key);
        match entry {
            Some(mut e) => match e.value_mut() {
                Entry {
                    data: DataType::AggregateType(AggregateType::SortedSet(sorted_set)),
                    ..
                } => Ok(f(sorted_set)),
                _ => Err("the value stored at key is not a sorted set.".to_owned()),
            },
            None => Ok(none_value()),
        }
    }
}

impl Slot {
    pub fn zadd(
        &self,
        key: String,
        values: Vec<Node>,
        nx_xx: NxXx,
        gt_lt: GtLt,
        ch: bool,
        incr: bool,
    ) -> Result<usize> {
        SortedSet::mut_process_exists_or_new(self, &key, |set| {
            Ok(set.add(values, nx_xx, gt_lt, ch, incr))
        })
    }
}
