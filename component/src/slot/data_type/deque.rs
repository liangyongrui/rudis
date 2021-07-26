use std::{
    collections::VecDeque,
    ops::{Deref, DerefMut},
};

use serde::{Deserialize, Serialize};

use crate::slot::data_type::SimpleType;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Deque {
    inner: VecDeque<SimpleType>,
}

impl Deque {
    pub fn new() -> Self {
        Deque {
            inner: VecDeque::new(),
        }
    }

    pub fn shape(&self, mut start: i64, mut stop: i64) -> (usize, usize) {
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

impl Default for Deque {
    fn default() -> Self {
        Self::new()
    }
}
impl Deref for Deque {
    type Target = VecDeque<SimpleType>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Deque {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
