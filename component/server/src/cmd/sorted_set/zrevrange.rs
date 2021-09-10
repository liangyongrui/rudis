use common::{connection::parse::frame::Frame, options::Limit};
use db::Db;
use macros::ParseFrames;

/// <https://redis.io/commands/zrevrange>
#[derive(Debug, ParseFrames)]
pub struct Zrevrange<'a> {
    pub key: &'a [u8],
    pub min: &'a str,
    pub max: &'a str,
    pub withscores: bool,
}

impl Zrevrange<'_> {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let cmd = dict::cmd::sorted_set::range_by_rank::Req {
            key: self.key,
            start: self.min.parse()?,
            stop: self.max.parse()?,
            limit: Limit::None,
            rev: true,
        };
        let response = db.sorted_set_range_by_rank(cmd)?;
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
