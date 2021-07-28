use std::ops::Bound;

use rpds::RedBlackTreeSetSync;

use crate::{
    slot::{
        data_type::{sorted_set::Node, CollectionType, DataType, Float},
        dict, SimpleType,
    },
    utils::BoundExt,
};

pub mod add;
pub mod range_by_lex;
pub mod range_by_rank;
pub mod range_by_score;
pub mod rank;
pub mod remove;
pub mod remove_by_lex_range;
pub mod remove_by_rank_range;
pub mod remove_by_score_range;

pub(self) fn get_value(
    key: &[u8],
    dict: &dict::Dict,
) -> crate::Result<Option<RedBlackTreeSetSync<Node>>> {
    if let Some(v) = dict.d_get(key) {
        if let DataType::CollectionType(CollectionType::SortedSet(ref sorted_set)) = v.data {
            Ok(Some(sorted_set.value.clone()))
        } else {
            Err("error type".into())
        }
    } else {
        Ok(None)
    }
}

pub(self) fn shape_limit(limit: Option<(usize, i64)>, len: usize) -> (usize, usize) {
    match limit {
        Some((offset, count)) => (offset, if count < 0 { len } else { count as usize }),
        None => (0, len),
    }
}

pub(self) fn shape_rank(mut start: i64, mut stop: i64, len: usize) -> (usize, usize) {
    let len = len as i64;
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

pub(self) fn bigger_range(range: (Bound<Float>, Bound<Float>)) -> (Bound<Node>, Bound<Node>) {
    (
        range.0.map(|f| Node {
            score: f,
            key: SimpleType::Null,
        }),
        range.1.map(|f| Node {
            score: f,
            key: SimpleType::Big,
        }),
    )
}
