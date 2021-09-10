use common::{
    connection::parse::frame::Frame,
    options::{Limit, RangeCmdOrder},
};
use db::Db;
use macros::ParseFrames;

/// <https://redis.io/commands/zrangebyscore>
#[derive(Debug, ParseFrames)]
pub struct Zrangebyscore<'a> {
    pub key: &'a [u8],
    pub min: &'a str,
    pub max: &'a str,
    pub withscores: bool,
    #[optional]
    pub limit: Limit,
}

impl Zrangebyscore<'_> {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let min = RangeCmdOrder::parse_float_bound(self.min)?;
        let max = RangeCmdOrder::parse_float_bound(self.max)?;
        let cmd = dict::cmd::sorted_set::range_by_score::Req {
            key: self.key,
            range: (min, max),
            limit: self.limit,
            rev: false,
        };
        let response = db.sorted_set_range_by_score(cmd)?;
        let mut res = vec![];
        if self.withscores {
            for n in response {
                res.push(Frame::OwnedSimple(n.key.into()));
                res.push(Frame::OwnedStringSimple(n.score.0.to_string()));
            }
        } else {
            for n in response {
                res.push(Frame::OwnedSimple(n.key.into()));
            }
        }
        Ok(Frame::Array(res))
    }
}
