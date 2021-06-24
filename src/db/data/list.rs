use std::{
    collections::VecDeque,
    ops::{Deref, DerefMut},
    usize,
};

use super::{bytes::Bytes, Data};
use crate::db::{result::Result, state::State};

/// redis list 中元素顺序 和  VecDeque 的内存顺序关系
/// L.....R
/// front.....back

#[derive(Debug, Clone)]
pub struct List(VecDeque<Bytes>);

impl Deref for List {
    type Target = VecDeque<Bytes>;

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
}
impl State {
    fn get_or_new_list(&mut self, key: String) -> Result<&mut List> {
        let entry = self.get_or_insert_entry(&key, || (Data::List(List(VecDeque::new())), None));
        match entry.data {
            Data::List(ref mut list) => Ok(list),
            _ => Err("the value stored at key is not a list.".to_owned()),
        }
    }

    fn get_list(&self, key: &str) -> Option<Result<&List>> {
        self.entries.get(key).map(|e| match e.data {
            Data::List(ref list) => Ok(list),
            _ => Err("the value stored at key is not a list.".to_owned()),
        })
    }

    fn get_list_mut(&mut self, key: &str) -> Option<Result<&mut List>> {
        self.entries.get_mut(key).map(|e| match e.data {
            Data::List(ref mut list) => Ok(list),
            _ => Err("the value stored at key is not a list.".to_owned()),
        })
    }

    pub(crate) fn lpushx(&mut self, key: &str, values: Vec<Bytes>) -> Result<usize> {
        match self.get_list_mut(key) {
            Some(r) => {
                let list = r?;
                for v in values {
                    list.push_front(v)
                }
                Ok(list.len())
            }
            None => Ok(0),
        }
    }

    pub(crate) fn rpushx(&mut self, key: &str, values: Vec<Bytes>) -> Result<usize> {
        match self.get_list_mut(key) {
            Some(r) => {
                let list = r?;
                for v in values {
                    list.push_back(v)
                }
                Ok(list.len())
            }
            None => Ok(0),
        }
    }
    pub(crate) fn lpush(&mut self, key: String, values: Vec<Bytes>) -> Result<usize> {
        let list = self.get_or_new_list(key)?;
        for v in values {
            list.push_front(v)
        }
        Ok(list.len())
    }

    pub(crate) fn rpush(&mut self, key: String, values: Vec<Bytes>) -> Result<usize> {
        let list = self.get_or_new_list(key)?;
        for v in values {
            list.push_back(v)
        }
        Ok(list.len())
    }

    pub(crate) fn lpop(&mut self, key: &str, count: usize) -> Result<Option<Vec<Bytes>>> {
        match self.get_list_mut(key) {
            Some(r) => {
                let list = r?;
                let mut res = vec![];
                for _ in 0..count {
                    if let Some(v) = list.pop_front() {
                        res.push(v)
                    } else {
                        break;
                    }
                }
                Ok(Some(res))
            }
            None => Ok(None),
        }
    }

    pub(crate) fn rpop(&mut self, key: &str, count: usize) -> Result<Option<Vec<Bytes>>> {
        match self.get_list_mut(key) {
            Some(r) => {
                let list = r?;
                let mut res = vec![];
                for _ in 0..count {
                    if let Some(v) = list.pop_back() {
                        res.push(v)
                    } else {
                        break;
                    }
                }
                Ok(Some(res))
            }
            None => Ok(None),
        }
    }

    pub(crate) fn lrange(&self, key: &str, start: i64, stop: i64) -> Result<Vec<Bytes>> {
        match self.get_list(key) {
            Some(e) => {
                let list = e?;
                let (begin, end) = list.shape(start, stop);
                let mut res = vec![];
                for i in begin..end {
                    res.push(list[i].clone())
                }
                Ok(res)
            }
            None => Ok(vec![]),
        }
    }

    pub(crate) fn llen(&self, key: &str) -> Result<usize> {
        match self.get_list(key) {
            Some(e) => Ok(e?.len()),
            None => Ok(0),
        }
    }
}
