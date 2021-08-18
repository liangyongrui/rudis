use std::{ops::Bound, sync::Arc};

use db::Db;
use rcc_macros::ParseFrames;

use crate::Frame;

/// https://redis.io/commands/zremrangebylex
#[derive(Debug, Clone, ParseFrames)]
pub struct Zremrangebylex {
    pub key: Arc<[u8]>,
    pub min: String,
    pub max: String,
}

impl From<Zremrangebylex> for dict::cmd::sorted_set::remove_by_lex_range::Req {
    fn from(old: Zremrangebylex) -> Self {
        let min = old.min;
        let max = old.max;
        let range = {
            let min = if min == "-" {
                Bound::Unbounded
            } else if let Some(s) = min.strip_prefix('(') {
                Bound::Excluded(s.into())
            } else {
                Bound::Included(min[1..].into())
            };
            let max = if max == "+" {
                Bound::Unbounded
            } else if let Some(s) = max.strip_prefix('(') {
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