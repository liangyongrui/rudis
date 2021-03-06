use db::Db;
use macros::ParseFrames;

use crate::Frame;

/// <https://redis.io/commands/zremrangebyrank>
#[derive(Debug, ParseFrames)]
pub struct Zremrangebyrank<'a> {
    pub key: &'a [u8],
    pub range: (i64, i64),
}

impl Zremrangebyrank<'_> {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let res =
            db.sorted_set_remove_by_rank_range(dict::cmd::sorted_set::remove_by_rank_range::Req {
                key: self.key.into(),
                start: self.range.0,
                stop: self.range.1,
                rev: false,
            })?;
        Ok(Frame::Integer(res.len() as _))
    }
}
