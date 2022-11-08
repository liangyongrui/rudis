use db::Db;
use macros::ParseFrames;

use crate::Frame;
/// <https://redis.io/commands/zrevrank>
#[derive(Debug, ParseFrames)]
pub struct Zrevrank<'a> {
    pub key: &'a [u8],
    pub member: &'a [u8],
}

impl Zrevrank<'_> {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let response = (db.sorted_set_rank(dict::cmd::sorted_set::rank::Req {
            key: self.key,
            member: self.member,
            rev: true,
        })?)
        .map_or(Frame::Null, |v| Frame::Integer(v as _));
        Ok(response)
    }
}
