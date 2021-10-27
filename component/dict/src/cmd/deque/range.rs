use std::vec;

use crate::{cmd::Read, data_type::DataType, Dict};

/// These offsets can also be negative numbers indicating offsets starting at the end of the list.
/// For example, -1 is the last element of the list, -2 the penultimate, and so on.
///
/// Out of range indexes will not produce an error. If start is larger than the end of the list,
/// an empty list is returned. If stop is larger than the actual end of the list,
/// It will treat it like the last element of the list.
#[derive(Debug, Clone)]
pub struct Req<'a> {
    pub key: &'a [u8],
    pub start: i64,
    pub stop: i64,
}

impl<'a, D: Dict> Read<Vec<DataType>, D> for Req<'a> {
    #[tracing::instrument(skip(dict), level = "debug")]
    fn apply(self, dict: &mut D) -> common::Result<Vec<DataType>> {
        if let Some(v) = dict.get(self.key) {
            return if let DataType::Deque(ref deque) = v.data {
                let (b, e) = deque.shape(self.start, self.stop);
                Ok(deque.range(b..e).cloned().collect())
            } else {
                Err("WRONGTYPE Operation against a key holding the wrong kind of value".into())
            };
        }
        Ok(vec![])
    }
}

// utest see push
