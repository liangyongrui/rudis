use common::{
    connection::parse::frame::Frame,
    options::{Limit, RangeCmdOrder},
};
use db::Db;
use macros::ParseFrames;

/// <https://redis.io/commands/zrangebylex>

#[derive(Debug, ParseFrames)]
pub struct Zrangebylex<'a> {
    pub key: &'a [u8],
    pub min: &'a str,
    pub max: &'a str,
    #[optional]
    pub limit: Limit,
}

impl Zrangebylex<'_> {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let min = RangeCmdOrder::parse_lex_bound(self.min)?;
        let max = RangeCmdOrder::parse_lex_bound(self.max)?;
        let cmd = dict::cmd::sorted_set::range_by_lex::Req {
            key: self.key,
            range: (min, max),
            limit: self.limit,
            rev: false,
        };
        let response = db.sorted_set_range_by_lex(cmd)?;
        let mut res = vec![];
        for n in response {
            res.push(Frame::OwnedSimple(n.key.into()));
        }
        Ok(Frame::Array(res))
    }
}
