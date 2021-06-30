use std::{
    collections::VecDeque,
    ops::{Deref, DerefMut},
    usize,
};

use super::{blob::Blob, AggregateType, DataType};
use crate::db::{
    result::Result,
    slot::{Entry, Slot},
};

/// redis list 中元素顺序 和  VecDeque 的内存顺序关系
/// L.....R
/// front.....back

#[derive(Debug, Clone)]
pub struct List(VecDeque<Blob>);

impl Deref for List {
    type Target = VecDeque<Blob>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for List {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl List {
    fn shape(&self, mut start: i64, mut stop: i64) -> (usize, usize) {
        let len = self.len() as i64;
        if start < 0 {
            start += len;
        }
        if stop < 0 {
            stop += len;
        }
        if start >= len || stop < 0 || stop < start {
            return (0, 0);
        }
        if start < 0 {
            start = 0
        }
        if stop >= len {
            stop = len - 1
        }
        (start as usize, stop as usize + 1)
    }
    fn process<T, F: FnOnce(&List) -> T>(
        slot: &Slot,
        key: &str,
        f: F,
        none_value: fn() -> T,
    ) -> Result<T> {
        match slot.entries.get(key) {
            Some(v) => match v.value() {
                Entry {
                    data: DataType::AggregateType(AggregateType::List(list)),
                    ..
                } => Ok(f(list)),
                _ => Err("the value stored at key is not a list.".to_owned()),
            },
            None => Ok(none_value()),
        }
    }

    fn mut_process<T, F: FnOnce(&mut List) -> T>(
        slot: &Slot,
        key: &str,
        f: F,
        none_value: fn() -> T,
    ) -> Result<T> {
        match slot.entries.get_mut(key) {
            Some(mut v) => match v.value_mut() {
                Entry {
                    data: DataType::AggregateType(AggregateType::List(list)),
                    ..
                } => Ok(f(list)),
                _ => Err("the value stored at key is not a list.".to_owned()),
            },
            None => Ok(none_value()),
        }
    }

    fn mut_process_exists_or_new<T, F: FnOnce(&mut List) -> T>(
        slot: &Slot,
        key: &str,
        f: F,
    ) -> Result<T> {
        let mut entry = slot.get_or_insert_entry(key, || {
            (
                DataType::AggregateType(AggregateType::List(List(VecDeque::new()))),
                None,
            )
        });
        match entry.value_mut() {
            Entry {
                data: DataType::AggregateType(AggregateType::List(list)),
                ..
            } => Ok(f(list)),
            _ => Err("the value stored at key is not a list.".to_owned()),
        }
    }
}

impl Slot {
    pub(crate) fn lpushx(&self, key: &str, values: Vec<Blob>) -> Result<usize> {
        List::mut_process(
            self,
            key,
            |list| {
                for v in values {
                    list.push_front(v)
                }
                list.len()
            },
            || 0,
        )
    }

    pub(crate) fn rpushx(&self, key: &str, values: Vec<Blob>) -> Result<usize> {
        List::mut_process(
            self,
            key,
            |list| {
                for v in values {
                    list.push_back(v)
                }
                list.len()
            },
            || 0,
        )
    }
    pub(crate) fn lpush(&self, key: &str, values: Vec<Blob>) -> Result<usize> {
        List::mut_process_exists_or_new(self, key, |list| {
            for v in values {
                list.push_front(v)
            }
            list.len()
        })
    }

    pub(crate) fn rpush(&self, key: String, values: Vec<Blob>) -> Result<usize> {
        List::mut_process_exists_or_new(self, &key, |list| {
            for v in values {
                list.push_back(v)
            }
            list.len()
        })
    }

    pub(crate) fn lpop(&self, key: &str, count: usize) -> Result<Option<Vec<Blob>>> {
        List::mut_process(
            self,
            key,
            |list| {
                let mut res = vec![];
                for _ in 0..count {
                    if let Some(v) = list.pop_front() {
                        res.push(v)
                    } else {
                        break;
                    }
                }
                Some(res)
            },
            || None,
        )
    }

    pub(crate) fn rpop(&self, key: &str, count: usize) -> Result<Option<Vec<Blob>>> {
        List::mut_process(
            self,
            key,
            |list| {
                let mut res = vec![];
                for _ in 0..count {
                    if let Some(v) = list.pop_back() {
                        res.push(v)
                    } else {
                        break;
                    }
                }
                Some(res)
            },
            || None,
        )
    }

    pub(crate) fn lrange(&self, key: &str, start: i64, stop: i64) -> Result<Vec<Blob>> {
        List::process(
            self,
            key,
            |list| {
                let (begin, end) = list.shape(start, stop);
                let mut res = vec![];
                for i in begin..end {
                    res.push(list[i].clone())
                }
                res
            },
            Vec::new,
        )
    }

    pub(crate) fn llen(&self, key: &str) -> Result<usize> {
        List::process(self, key, |list| list.len(), || 0)
    }
}
