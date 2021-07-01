use std::{
    collections::HashMap,
    ops::{Bound, RangeBounds},
    sync::Arc,
};

use rpds::RedBlackTreeSetSync;
use tracing::debug;

use super::{AggregateType, DataType};
use crate::{
    db::{
        result::Result,
        slot::{Entry, Slot},
    },
    options::{GtLt, NxXx},
    utils::BoundExt,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    key: String,
    score: f64,
}

pub enum RangeItem {
    Rank((Bound<i64>, Bound<i64>)),
    Socre((Bound<f64>, Bound<f64>)),
    Lex((Bound<String>, Bound<String>)),
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

fn zrange_by_rank(
    set: RedBlackTreeSetSync<Node>,
    range: (Bound<i64>, Bound<i64>),
    rev: bool,
    limit: Option<(i64, i64)>,
) -> Vec<Node> {
    todo!()
}

fn zrange_by_score(
    set: RedBlackTreeSetSync<Node>,
    range: (Bound<f64>, Bound<f64>),
    rev: bool,
    limit: Option<(i64, i64)>,
) -> Vec<Node> {
    let set_range: (Bound<Node>, Bound<Node>) = if rev {
        (
            range.1.map(|t| Node {
                key: "".to_owned(),
                score: t - 0.1,
            }),
            range.0.map(|t| Node {
                key: "".to_owned(),
                score: t,
            }),
        )
    } else {
        (
            range.0.map(|t| Node {
                key: "".to_owned(),
                score: t,
            }),
            range.1.map(|t| Node {
                key: "".to_owned(),
                score: t + 0.1,
            }),
        )
    };
    debug!(?range, ?set_range);
    let (offset, count) = match limit {
        Some((offset, count)) => (
            if offset < 0 { 0 } else { offset as usize },
            if count < 0 {
                set.size()
            } else {
                count as usize
            },
        ),
        None => (0, set.size()),
    };
    if rev {
        set.range(set_range)
            .rev()
            .filter(|t| range.contains(&t.score))
            .take(count)
            .skip(offset)
            .cloned()
            .collect()
    } else {
        set.range(set_range)
            .filter(|t| range.contains(&t.score))
            .take(count)
            .skip(offset)
            .cloned()
            .collect()
    }
}

fn zrange_by_lex(
    set: RedBlackTreeSetSync<Node>,
    range: (Bound<String>, Bound<String>),
    rev: bool,
    limit: Option<(i64, i64)>,
) -> Vec<Node> {
    todo!()
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

    pub fn zrange(
        &self,
        key: &str,
        range: RangeItem,
        rev: bool,
        limit: Option<(i64, i64)>,
    ) -> Result<Vec<Node>> {
        let res = SortedSet::process(self, &key, |set| Some((*set.value).clone()), || None)?;
        match res {
            Some(tree_set) => Ok(match range {
                RangeItem::Rank(range) => zrange_by_rank(tree_set, range, rev, limit),
                RangeItem::Socre(range) => zrange_by_score(tree_set, range, rev, limit),
                RangeItem::Lex(range) => zrange_by_lex(tree_set, range, rev, limit),
            }),
            None => Ok(vec![]),
        }
    }
}

#[test]
fn test_tree() {
    let mut tree = RedBlackTreeSetSync::new_sync();
    tree.insert_mut(1);
    tree.insert_mut(5);
    tree.insert_mut(7);
    tree.insert_mut(2);
    tree.iter().for_each(|t| println!("{}", t));
    tree.range(2..6).for_each(|t| println!("{}", t));
}

#[test]
fn test_iter() {
    let v = vec![1, 2, 3, 4, 5, 6, 7];
    let v2: Vec<_> = v.into_iter().filter(|t| *t > 2).take(4).skip(1).collect();
    assert_eq!(v2, vec![4, 5, 6]);
}
