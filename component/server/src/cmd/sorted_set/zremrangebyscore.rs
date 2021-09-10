use common::{connection::parse::frame::Frame, options::RangeCmdOrder};
use db::Db;
use macros::ParseFrames;

/// <https://redis.io/commands/zremrangebyscore>
#[derive(Debug, ParseFrames)]
pub struct Zremrangebyscore<'a> {
    pub key: &'a [u8],
    pub min: &'a str,
    pub max: &'a str,
}

impl Zremrangebyscore<'_> {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let min = RangeCmdOrder::parse_float_bound(self.min)?;
        let max = RangeCmdOrder::parse_float_bound(self.max)?;
        let res = db.sorted_set_remove_by_score_range(
            dict::cmd::sorted_set::remove_by_score_range::Req {
                key: self.key.into(),
                range: (min, max),
                rev: false,
            },
        )?;
        Ok(Frame::Integer(res.len() as _))
    }
}
