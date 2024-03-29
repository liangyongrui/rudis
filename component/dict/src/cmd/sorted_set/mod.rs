use std::ops::Bound;

use common::{options::Limit, BoundExt};

use crate::data_type::{sorted_set::Node, Float};

pub mod add;
pub mod range_by_lex;
pub mod range_by_rank;
pub mod range_by_score;
pub mod rank;
pub mod remove;
pub mod remove_by_lex_range;
pub mod remove_by_rank_range;
pub mod remove_by_score_range;

pub(self) fn shape_limit(limit: Limit, len: usize) -> (usize, usize) {
    match limit {
        Limit::Limit(mut offset, count) => {
            if offset < 0 {
                offset = 0.max(len as i64 - offset);
            }
            (
                offset as usize,
                if count < 0 { len } else { count as usize },
            )
        }
        Limit::None => (0, len),
    }
}

pub(self) const fn shape_rank(mut start: i64, mut stop: i64, len: usize) -> (usize, usize) {
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
        start = 0;
    }
    if stop >= len {
        stop = len - 1;
    }
    (start as usize, stop as usize + 1)
}

pub(self) fn bigger_range(range: (Bound<Float>, Bound<Float>)) -> (Bound<Node>, Bound<Node>) {
    (
        range.0.map(|f| Node {
            score: f,
            key: [][..].into(),
        }),
        range.1.map(|f| Node {
            score: Float(f.0 + 0.1),
            key: [][..].into(),
        }),
    )
}
