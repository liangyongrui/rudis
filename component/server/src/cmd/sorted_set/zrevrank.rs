use std::sync::Arc;

use db::Db;
use rcc_macros::ParseFrames;

use crate::Frame;
/// https://redis.io/commands/zrevrank
#[derive(Debug, ParseFrames)]
pub struct Zrevrank {
    pub key: Arc<[u8]>,
    pub member: String,
}
impl<'a> From<&'a Zrevrank> for dict::cmd::sorted_set::rank::Req<'a> {
    fn from(old: &'a Zrevrank) -> Self {
        Self {
            key: &old.key,
            member: &old.member,
            rev: true,
        }
    }
}

impl Zrevrank {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let response = match db.sorted_set_rank((&self).into())? {
            None => Frame::Null,
            Some(v) => Frame::Integer(v as _),
        };
        Ok(response)
    }
}