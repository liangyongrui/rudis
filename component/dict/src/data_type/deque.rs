use std::{
    collections::VecDeque,
    ops::{Deref, DerefMut},
};

use serde::{Deserialize, Serialize};

use crate::data_type::DataType;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Deque {
    inner: VecDeque<DataType>,
}

impl Deque {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    #[must_use]
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
            start = 0;
        }
        if stop >= len {
            stop = len - 1;
        }
        (start as usize, stop as usize + 1)
    }
}

impl Deref for Deque {
    type Target = VecDeque<DataType>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Deque {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
