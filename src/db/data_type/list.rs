use std::{
    collections::VecDeque,
    ops::{Deref, DerefMut},
    usize,
};

use super::{AggregateType, DataType, SimpleType};
use crate::db::{
    result::Result,
    slot::{Entry, Slot},
};

/// redis list 中元素顺序 和  VecDeque 的内存顺序关系
/// L.....R
/// front.....back

#[derive(Debug, Clone)]
pub struct List(VecDeque<SimpleType>);

impl Deref for List {
    type Target = VecDeque<SimpleType>;

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
        key: &SimpleType,
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
        key: &SimpleType,
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
        key: SimpleType,
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
    pub(crate) fn lpushx(&self, key: &SimpleType, values: Vec<SimpleType>) -> Result<usize> {
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

    pub(crate) fn rpushx(&self, key: &SimpleType, values: Vec<SimpleType>) -> Result<usize> {
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

    pub(crate) fn lpush(&self, key: SimpleType, values: Vec<SimpleType>) -> Result<usize> {
        List::mut_process_exists_or_new(self, key, |list| {
            for v in values {
                list.push_front(v)
            }
            list.len()
        })
    }

    pub(crate) fn rpush(&self, key: SimpleType, values: Vec<SimpleType>) -> Result<usize> {
        List::mut_process_exists_or_new(self, key, |list| {
            for v in values {
                list.push_back(v)
            }
            list.len()
        })
    }

    pub(crate) fn lpop(&self, key: &SimpleType, count: usize) -> Result<Option<Vec<SimpleType>>> {
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

    pub(crate) fn rpop(&self, key: &SimpleType, count: usize) -> Result<Option<Vec<SimpleType>>> {
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

    pub(crate) fn lrange(
        &self,
        key: &SimpleType,
        start: i64,
        stop: i64,
    ) -> Result<Vec<SimpleType>> {
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

    pub(crate) fn llen(&self, key: &SimpleType) -> Result<usize> {
        List::process(self, key, |list| list.len(), || 0)
    }
}

#[cfg(test)]
mod test {
    use crate::db::slot::Slot;

    #[test]
    fn test_shape() {
        let list = super::List(
            vec![
                1.into(),
                2.into(),
                3.into(),
                4.into(),
                5.into(),
                6.into(),
                7.into(),
                8.into(),
                9.into(),
                10.into(),
            ]
            .into_iter()
            .collect(),
        );
        assert_eq!(list.shape(1, 5), (1, 6));
        assert_eq!(list.shape(1, 100), (1, 10));
        assert_eq!(list.shape(0, 100), (0, 10));
        assert_eq!(list.shape(-2, 100), (8, 10));
        assert_eq!(list.shape(-2, -2), (8, 9));
        assert_eq!(list.shape(-2, -100), (0, 0));
        assert_eq!(list.shape(-2, -10), (0, 0));
        assert_eq!(list.shape(2, -10), (0, 0));
        assert_eq!(list.shape(2, -1), (2, 10));
        assert_eq!(list.shape(-100, -1), (0, 10));
    }

    #[tokio::test]
    async fn test() {
        let slot = Slot::new();
        assert_eq!(
            slot.lpush(
                "key".into(),
                vec![
                    "123".into(),
                    "4".into(),
                    5.into(),
                    6.into(),
                    7.into(),
                    8.into(),
                    9.into()
                ]
            ),
            Ok(7)
        );
        assert_eq!(
            slot.lpop(&"key".into(), 2),
            Ok(Some(vec![9.into(), 8.into()]))
        );
        assert_eq!(
            slot.rpop(&"key".into(), 2),
            Ok(Some(vec!["123".into(), "4".into()]))
        );
        assert_eq!(
            slot.rpush("key".into(), vec![7.into(), 8.into(), 9.into()]),
            Ok(6)
        );
        assert_eq!(
            slot.rpop(&"key".into(), 2),
            Ok(Some(vec![9.into(), 8.into()]))
        );
        assert_eq!(
            slot.lrange(&"key".into(), 0, 10),
            Ok(vec![7.into(), 6.into(), 5.into(), 7.into()])
        );
        assert_eq!(slot.llen(&"key".into()), Ok(4));
        assert_eq!(slot.lpushx(&"key2".into(), vec![9.into()]), Ok(0));
        assert_eq!(slot.rpushx(&"key2".into(), vec![9.into()]), Ok(0));
        assert_eq!(slot.llen(&"key2".into()), Ok(0));
        assert_eq!(slot.rpushx(&"key".into(), vec![9.into()]), Ok(5));
        assert_eq!(slot.llen(&"key".into()), Ok(5));
    }
}
