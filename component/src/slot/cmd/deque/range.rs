use std::vec;

use parking_lot::RwLock;

use crate::slot::{cmd::Read, data_type::DataType, dict::Dict};

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

impl<'a> Read<Vec<DataType>> for Req<'a> {
    #[tracing::instrument(skip(dict), level = "debug")]
    fn apply(self, dict: &RwLock<Dict>) -> crate::Result<Vec<DataType>> {
        if let Some(v) = dict.read().d_get(self.key) {
            return if let DataType::Deque(ref deque) = v.data {
                let (b, e) = deque.shape(self.start, self.stop);
                Ok(deque.range(b..e).cloned().collect())
            } else {
                Err("error type".into())
            };
        }
        Ok(vec![])
    }
}

// utest see push
