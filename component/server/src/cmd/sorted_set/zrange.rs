use common::{
    connection::parse::frame::Frame,
    options::{Limit, RangeCmdOrder},
};
use db::Db;
use macros::ParseFrames;

#[derive(Debug, ParseFrames)]
pub struct Zrange<'a> {
    pub key: &'a [u8],
    pub min: &'a str,
    pub max: &'a str,
    #[optional]
    pub order: RangeCmdOrder,
    pub rev: bool,
    #[optional]
    pub limit: Limit,
    pub withscores: bool,
}

impl Zrange<'_> {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let response = match self.order {
            RangeCmdOrder::Byscore => {
                let min = RangeCmdOrder::parse_float_bound(self.min)?;
                let max = RangeCmdOrder::parse_float_bound(self.max)?;
                let cmd = dict::cmd::sorted_set::range_by_score::Req {
                    key: self.key,
                    range: if self.rev { (max, min) } else { (min, max) },
                    limit: self.limit,
                    rev: self.rev,
                };
                db.sorted_set_range_by_score(cmd)?
            }
            RangeCmdOrder::Bylex => {
                let min = RangeCmdOrder::parse_lex_bound(self.min)?;
                let max = RangeCmdOrder::parse_lex_bound(self.max)?;
                let cmd = dict::cmd::sorted_set::range_by_lex::Req {
                    key: self.key,
                    range: if self.rev { (max, min) } else { (min, max) },
                    limit: self.limit,
                    rev: self.rev,
                };
                db.sorted_set_range_by_lex(cmd)?
            }
            RangeCmdOrder::Byrank => {
                let cmd = dict::cmd::sorted_set::range_by_rank::Req {
                    key: self.key,
                    start: self.min.parse()?,
                    stop: self.max.parse()?,
                    limit: self.limit,
                    rev: self.rev,
                };
                db.sorted_set_range_by_rank(cmd)?
            }
        };

        let mut res = vec![];
        for n in response {
            res.push(Frame::OwnedSimple(n.key.into()));
            if self.withscores {
                res.push(Frame::OwnedStringSimple(n.score.0.to_string()));
            }
        }
        Ok(Frame::Array(res))
    }
}
