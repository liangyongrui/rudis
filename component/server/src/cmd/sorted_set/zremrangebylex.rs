use std::ops::Bound;

use db::Db;
use keys::Key;
use macros::ParseFrames;

use crate::Frame;

/// https://redis.io/commands/zremrangebylex
#[derive(Debug, Clone, ParseFrames)]
pub struct Zremrangebylex {
    pub key: Key,
    pub min: Box<[u8]>,
    pub max: Box<[u8]>,
}

impl From<Zremrangebylex> for dict::cmd::sorted_set::remove_by_lex_range::Req {
    fn from(old: Zremrangebylex) -> Self {
        let min = old.min;
        let max = old.max;
        let range = {
            let min = if min == b"-"[..].into() {
                Bound::Unbounded
            } else if let Some(s) = min.strip_prefix(b"(") {
                Bound::Excluded(s.into())
            } else {
                Bound::Included(min[1..].into())
            };
            let max = if max == b"+"[..].into() {
                Bound::Unbounded
            } else if let Some(s) = max.strip_prefix(b"(") {
                Bound::Excluded(s.into())
            } else {
                Bound::Included(max[1..].into())
            };
            (min, max)
        };
        Self {
            key: old.key,
            rev: false,
            range,
        }
    }
}

impl Zremrangebylex {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let res = db.sorted_set_remove_by_lex_range(self.into())?;
        Ok(Frame::Integer(res.len() as _))
    }
}
