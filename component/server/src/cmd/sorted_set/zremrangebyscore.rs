use std::ops::Bound;

use common::{float::Float, BoundExt};
use connection::parse::{frame::Frame, Parse};
use db::Db;
use keys::Key;

/// https://redis.io/commands/zremrangebyscore
#[derive(Debug, Clone)]
pub struct Zremrangebyscore {
    pub key: Key,
    pub range: (Bound<f64>, Bound<f64>),
}

impl From<Zremrangebyscore> for dict::cmd::sorted_set::remove_by_score_range::Req {
    fn from(old: Zremrangebyscore) -> Self {
        Self {
            key: old.key,
            rev: false,
            range: (old.range.0.map(Float), old.range.1.map(Float)),
        }
    }
}

impl Zremrangebyscore {
    pub fn parse_frames(parse: &mut Parse) -> common::Result<Self> {
        let key = parse.next_key()?;
        let min = parse.next_string()?;
        let max = parse.next_string()?;
        let range = {
            let min = if min == "-inf" {
                Bound::Unbounded
            } else if let Some(s) = min.strip_prefix('(') {
                Bound::Excluded(s.parse::<f64>()?)
            } else {
                Bound::Included(min.parse::<f64>()?)
            };
            let max = if max == "+inf" {
                Bound::Unbounded
            } else if let Some(s) = max.strip_prefix('(') {
                Bound::Excluded(s.parse::<f64>()?)
            } else {
                Bound::Included(max.parse::<f64>()?)
            };
            (min, max)
        };
        Ok(Self { key, range })
    }

    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let res = db.sorted_set_remove_by_score_range(self.into())?;
        Ok(Frame::Integer(res.len() as _))
    }
}
