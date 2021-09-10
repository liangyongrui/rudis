use common::{options::RangeCmdOrder, BoundExt};
use db::Db;
use macros::ParseFrames;

use crate::Frame;

/// <https://redis.io/commands/zremrangebylex>
#[derive(Debug, ParseFrames)]
pub struct Zremrangebylex<'a> {
    pub key: &'a [u8],
    pub min: &'a str,
    pub max: &'a str,
}

impl Zremrangebylex<'_> {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let min = RangeCmdOrder::parse_lex_bound(self.min)?;
        let max = RangeCmdOrder::parse_lex_bound(self.max)?;
        let res =
            db.sorted_set_remove_by_lex_range(dict::cmd::sorted_set::remove_by_lex_range::Req {
                key: self.key.into(),
                range: (min.map(|t| t.into()), max.map(|t| t.into())),
                rev: false,
            })?;
        Ok(Frame::Integer(res.len() as _))
    }
}
