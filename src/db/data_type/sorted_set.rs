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
    utils::{
        options::{GtLt, NxXx},
        BoundExt,
    },
};

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub key: String,
    pub score: f64,
}

impl Node {
    pub fn new(key: String, score: f64) -> Self {
        Self { key, score }
    }
}

#[derive(Debug)]
pub enum RangeItem {
    Rank((i64, i64)),
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
    fn add(&mut self, nodes: Vec<Node>, nx_xx: NxXx, gt_lt: GtLt, ch: bool, incr: bool) -> usize {
        let old_len = self.value.size();
        let mut new = (*self.value).clone();
        for mut v in nodes {
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

    pub fn zrem(&mut self, members: Vec<String>) -> usize {
        let old_len = self.value.size();
        let mut set = (*self.value).clone();
        for v in members {
            if let Some(n) = self.hash.remove(&v) {
                set.remove_mut(&n);
            }
        }
        self.version += 1;
        self.value = Arc::new(set);
        self.value.size() - old_len
    }

    pub fn zremrange_by_rank(&mut self, range: (i64, i64)) -> usize {
        let mut n = (*self.value).clone();
        let old_size = n.size();
        let range = n.zrange_by_rank(range, false, None);
        for i in range {
            n.remove_mut(&i);
            self.hash.remove(&i.key);
        }
        self.value = Arc::new(n);
        old_size - self.value.size()
    }

    pub fn zremrange_by_score(&mut self, range: (Bound<f64>, Bound<f64>)) -> usize {
        let mut n = (*self.value).clone();
        let old_size = n.size();
        let range = n.zrange_by_score(range, false, None);
        for i in range {
            n.remove_mut(&i);
            self.hash.remove(&i.key);
        }
        self.value = Arc::new(n);
        old_size - self.value.size()
    }
}
/// The indexes can also be negative numbers indicating offsets from the end of the sorted set,
/// with -1 being the last element of the sorted set, -2 the penultimate element, and so on.
///
/// Out of range indexes do not produce an error.
///
/// If <min> is greater than either the end index of the sorted set or <max>, an empty list is returned.
///
/// If <max> is greater than the end index of the sorted set, Redis will use the last element of the sorted set.
fn shape_rank_index(len: usize, mut index: i64, max: bool) -> usize {
    if index < 0 {
        index += len as i64;
    }
    if max {
        index += 1;
    }
    if index < 0 {
        index = 0;
    }
    let mut index = index as usize;
    if index > len {
        index = len
    }
    index
}

trait TreeSetExt {
    fn zrank(&self, value: &Node, rev: bool) -> Option<usize>;

    fn zrange_by_rank(&self, range: (i64, i64), rev: bool, limit: Option<(i64, i64)>) -> Vec<Node>;

    fn zrange_by_score(
        &self,
        range: (Bound<f64>, Bound<f64>),
        rev: bool,
        limit: Option<(i64, i64)>,
    ) -> Vec<Node>;

    fn zrange_by_lex(
        &self,
        range: (Bound<String>, Bound<String>),
        rev: bool,
        limit: Option<(i64, i64)>,
    ) -> Vec<Node>;
}
impl TreeSetExt for RedBlackTreeSetSync<Node> {
    fn zrange_by_rank(&self, range: (i64, i64), rev: bool, limit: Option<(i64, i64)>) -> Vec<Node> {
        let range = (
            shape_rank_index(self.size(), range.0, false),
            shape_rank_index(self.size(), range.1, true),
        );
        let range = range.0..range.1;
        let (offset, count) = sharp_limit(limit, self.size());
        if rev {
            self.iter()
                .rev()
                .enumerate()
                .filter(|(index, _)| range.contains(index))
                .take(count)
                .skip(offset)
                .map(|(_, node)| node.clone())
                .collect()
        } else {
            self.iter()
                .enumerate()
                .filter(|(index, _)| range.contains(index))
                .take(count)
                .skip(offset)
                .map(|(_, node)| node.clone())
                .collect()
        }
    }

    fn zrange_by_score(
        &self,
        mut range: (Bound<f64>, Bound<f64>),
        rev: bool,
        limit: Option<(i64, i64)>,
    ) -> Vec<Node> {
        if rev {
            range = (range.1, range.0)
        }
        let set_range: (Bound<Node>, Bound<Node>) = (
            range.0.map(|t| Node {
                key: "".to_owned(),
                score: t,
            }),
            range.1.map(|t| Node {
                key: "".to_owned(),
                score: t + 0.1,
            }),
        );
        debug!(?range, ?set_range);
        let (offset, count) = sharp_limit(limit, self.size());
        if rev {
            self.range(set_range)
                .rev()
                .filter(|t| range.contains(&t.score))
                .take(count)
                .skip(offset)
                .cloned()
                .collect()
        } else {
            self.range(set_range)
                .filter(|t| range.contains(&t.score))
                .take(count)
                .skip(offset)
                .cloned()
                .collect()
        }
    }

    fn zrange_by_lex(
        &self,
        mut range: (Bound<String>, Bound<String>),
        rev: bool,
        limit: Option<(i64, i64)>,
    ) -> Vec<Node> {
        let score = if let Some(t) = self.first() {
            t.score
        } else {
            return vec![];
        };
        if rev {
            range = (range.1, range.0)
        }
        let set_range: (Bound<Node>, Bound<Node>) = (
            range.0.map(|key| Node { key, score }),
            range.1.map(|key| Node { key, score }),
        );
        let (offset, count) = sharp_limit(limit, self.size());
        debug!(offset, count);
        if rev {
            self.range(set_range)
                .rev()
                .take(count)
                .skip(offset)
                .cloned()
                .collect()
        } else {
            self.range(set_range)
                .take(count)
                .skip(offset)
                .cloned()
                .collect()
        }
    }

    fn zrank(&self, value: &Node, rev: bool) -> Option<usize> {
        let ans = 0;
        if rev {
            for n in self.iter().rev() {
                if n.key == value.key {
                    return Some(ans);
                }
                if n.score < value.score {
                    return None;
                }
            }
        } else {
            for n in self.iter() {
                if n.key == value.key {
                    return Some(ans);
                }
                if n.score > value.score {
                    return None;
                }
            }
        }
        None
    }
}

fn sharp_limit(limit: Option<(i64, i64)>, len: usize) -> (usize, usize) {
    let (offset, count) = match limit {
        Some((mut offset, count)) => {
            if offset < 0 {
                offset = 0;
            }
            (
                offset as usize,
                if count < 0 {
                    len
                } else {
                    (offset + count) as usize
                },
            )
        }
        None => (0, len),
    };
    (offset, count)
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
        nodes: Vec<Node>,
        nx_xx: NxXx,
        gt_lt: GtLt,
        ch: bool,
        incr: bool,
    ) -> Result<usize> {
        SortedSet::mut_process_exists_or_new(self, &key, |set| {
            Ok(set.add(nodes, nx_xx, gt_lt, ch, incr))
        })
    }

    pub fn zrem(&self, key: &str, members: Vec<String>) -> Result<usize> {
        SortedSet::mut_process(self, key, |set| set.zrem(members), || 0)
    }

    pub fn zrank(&self, key: &str, member: &str, rev: bool) -> Result<Option<usize>> {
        SortedSet::process(
            self,
            key,
            |set| set.get(member).and_then(|n| set.value.zrank(n, rev)),
            || None,
        )
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
                RangeItem::Rank(range) => tree_set.zrange_by_rank(range, rev, limit),
                RangeItem::Socre(range) => tree_set.zrange_by_score(range, rev, limit),
                RangeItem::Lex(range) => tree_set.zrange_by_lex(range, rev, limit),
            }),
            None => Ok(vec![]),
        }
    }

    pub fn zremrange_by_rank(&self, key: &str, range: (i64, i64)) -> Result<usize> {
        SortedSet::mut_process(self, key, |set| set.zremrange_by_rank(range), || 0)
    }

    pub fn zremrange_by_score(&self, key: &str, range: (Bound<f64>, Bound<f64>)) -> Result<usize> {
        SortedSet::mut_process(self, key, |set| set.zremrange_by_score(range), || 0)
    }
}
#[cfg(test)]
mod test {

    use rpds::{rbt_set_sync, RedBlackTreeSetSync};

    use super::*;

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

    #[test]
    fn test_zrange_by_score() {
        let set: RedBlackTreeSetSync<Node> = rbt_set_sync![
            Node::new("n".to_owned(), 11.0),
            Node::new("m".to_owned(), 10.0),
            Node::new("l".to_owned(), 1.0),
            Node::new("k".to_owned(), 9.0),
            Node::new("j".to_owned(), 8.0),
            Node::new("i".to_owned(), 7.0),
            Node::new("h".to_owned(), 6.0),
            Node::new("g".to_owned(), 5.0),
            Node::new("f".to_owned(), 2.0),
            Node::new("e".to_owned(), 2.0),
            Node::new("e2".to_owned(), 2.0),
            Node::new("d".to_owned(), 2.0),
            Node::new("c".to_owned(), 1.0),
            Node::new("b".to_owned(), 1.0),
            Node::new("a".to_owned(), 1.0)
        ];
        let range = (Bound::Excluded(1.0), Bound::Included(10.0));
        let res: Vec<_> = set.zrange_by_score(range, false, None);
        assert_eq!(
            res,
            vec![
                Node::new("d".to_owned(), 2.0),
                Node::new("e".to_owned(), 2.0),
                Node::new("e2".to_owned(), 2.0),
                Node::new("f".to_owned(), 2.0),
                Node::new("g".to_owned(), 5.0),
                Node::new("h".to_owned(), 6.0),
                Node::new("i".to_owned(), 7.0),
                Node::new("j".to_owned(), 8.0),
                Node::new("k".to_owned(), 9.0),
                Node::new("m".to_owned(), 10.0),
            ]
        );
        let range = (Bound::Excluded(10.0), Bound::Excluded(1.0));
        let res: Vec<_> = set.zrange_by_score(range, true, None);
        assert_eq!(
            res,
            vec![
                Node::new("k".to_owned(), 9.0),
                Node::new("j".to_owned(), 8.0),
                Node::new("i".to_owned(), 7.0),
                Node::new("h".to_owned(), 6.0),
                Node::new("g".to_owned(), 5.0),
                Node::new("f".to_owned(), 2.0),
                Node::new("e2".to_owned(), 2.0),
                Node::new("e".to_owned(), 2.0),
                Node::new("d".to_owned(), 2.0),
            ]
        );

        let range = (Bound::Excluded(10.0), Bound::Excluded(1.0));
        let res: Vec<_> = set.zrange_by_score(range, true, Some((2, 6)));
        assert_eq!(
            res,
            vec![
                Node::new("i".to_owned(), 7.0),
                Node::new("h".to_owned(), 6.0),
                Node::new("g".to_owned(), 5.0),
                Node::new("f".to_owned(), 2.0),
                Node::new("e2".to_owned(), 2.0),
                Node::new("e".to_owned(), 2.0),
            ]
        );
    }

    #[test]
    fn test_zrange_by_rank() {
        let set: RedBlackTreeSetSync<Node> = rbt_set_sync![
            Node::new("n".to_owned(), 11.0),
            Node::new("m".to_owned(), 10.0),
            Node::new("l".to_owned(), 1.0),
            Node::new("k".to_owned(), 9.0),
            Node::new("j".to_owned(), 8.0),
            Node::new("i".to_owned(), 7.0),
            Node::new("h".to_owned(), 6.0),
            Node::new("g".to_owned(), 5.0),
            Node::new("f".to_owned(), 2.0),
            Node::new("e".to_owned(), 2.0),
            Node::new("e2".to_owned(), 2.0),
            Node::new("d".to_owned(), 2.0),
            Node::new("c".to_owned(), 1.0),
            Node::new("b".to_owned(), 1.0),
            Node::new("a".to_owned(), 1.0)
        ];
        let res: Vec<_> = set.zrange_by_rank((1, 10), false, None);
        assert_eq!(
            res,
            vec![
                Node::new("b".to_owned(), 1.0),
                Node::new("c".to_owned(), 1.0),
                Node::new("l".to_owned(), 1.0),
                Node::new("d".to_owned(), 2.0),
                Node::new("e".to_owned(), 2.0),
                Node::new("e2".to_owned(), 2.0),
                Node::new("f".to_owned(), 2.0),
                Node::new("g".to_owned(), 5.0),
                Node::new("h".to_owned(), 6.0),
                Node::new("i".to_owned(), 7.0),
            ]
        );
        let res: Vec<_> = set.zrange_by_rank((1, 10), true, None);
        assert_eq!(
            res,
            vec![
                Node::new("m".to_owned(), 10.0),
                Node::new("k".to_owned(), 9.0),
                Node::new("j".to_owned(), 8.0),
                Node::new("i".to_owned(), 7.0),
                Node::new("h".to_owned(), 6.0),
                Node::new("g".to_owned(), 5.0),
                Node::new("f".to_owned(), 2.0),
                Node::new("e2".to_owned(), 2.0),
                Node::new("e".to_owned(), 2.0),
                Node::new("d".to_owned(), 2.0),
            ]
        );

        let res: Vec<_> = set.zrange_by_rank((1, 10), true, Some((2, 6)));
        assert_eq!(
            res,
            vec![
                Node::new("j".to_owned(), 8.0),
                Node::new("i".to_owned(), 7.0),
                Node::new("h".to_owned(), 6.0),
                Node::new("g".to_owned(), 5.0),
                Node::new("f".to_owned(), 2.0),
                Node::new("e2".to_owned(), 2.0),
            ]
        );
    }

    #[test]
    fn test_zrange_by_lex() {
        let set: RedBlackTreeSetSync<Node> = rbt_set_sync![
            Node::new("n".to_owned(), 1.0),
            Node::new("m".to_owned(), 1.0),
            Node::new("l".to_owned(), 1.0),
            Node::new("k".to_owned(), 1.0),
            Node::new("j".to_owned(), 1.0),
            Node::new("i".to_owned(), 1.0),
            Node::new("h".to_owned(), 1.0),
            Node::new("g".to_owned(), 1.0),
            Node::new("f".to_owned(), 1.0),
            Node::new("e".to_owned(), 1.0),
            Node::new("e2".to_owned(), 1.0),
            Node::new("d".to_owned(), 1.0),
            Node::new("c".to_owned(), 1.0),
            Node::new("b".to_owned(), 1.0),
            Node::new("a".to_owned(), 1.0)
        ];
        let res = set.zrange_by_lex(
            (
                Bound::Excluded("a".to_owned()),
                Bound::Excluded("j".to_owned()),
            ),
            false,
            None,
        );
        assert_eq!(
            res,
            vec![
                Node::new("b".to_owned(), 1.0),
                Node::new("c".to_owned(), 1.0),
                Node::new("d".to_owned(), 1.0),
                Node::new("e".to_owned(), 1.0),
                Node::new("e2".to_owned(), 1.0),
                Node::new("f".to_owned(), 1.0),
                Node::new("g".to_owned(), 1.0),
                Node::new("h".to_owned(), 1.0),
                Node::new("i".to_owned(), 1.0),
            ]
        );
        let res = set.zrange_by_lex(
            (
                Bound::Excluded("j".to_owned()),
                Bound::Included("d".to_owned()),
            ),
            true,
            None,
        );
        assert_eq!(
            res,
            vec![
                Node::new("i".to_owned(), 1.0),
                Node::new("h".to_owned(), 1.0),
                Node::new("g".to_owned(), 1.0),
                Node::new("f".to_owned(), 1.0),
                Node::new("e2".to_owned(), 1.0),
                Node::new("e".to_owned(), 1.0),
                Node::new("d".to_owned(), 1.0),
            ]
        );

        let res = set.zrange_by_lex(
            (
                Bound::Excluded("j".to_owned()),
                Bound::Included("d".to_owned()),
            ),
            true,
            Some((2, 4)),
        );
        assert_eq!(
            res,
            vec![
                Node::new("g".to_owned(), 1.0),
                Node::new("f".to_owned(), 1.0),
                Node::new("e2".to_owned(), 1.0),
                Node::new("e".to_owned(), 1.0),
            ]
        );

        let res = set.zrange_by_lex(
            (Bound::Unbounded, Bound::Included("i".to_owned())),
            false,
            Some((2, 6)),
        );
        assert_eq!(
            res,
            vec![
                Node::new("c".to_owned(), 1.0),
                Node::new("d".to_owned(), 1.0),
                Node::new("e".to_owned(), 1.0),
                Node::new("e2".to_owned(), 1.0),
                Node::new("f".to_owned(), 1.0),
                Node::new("g".to_owned(), 1.0),
            ]
        );
    }
}
