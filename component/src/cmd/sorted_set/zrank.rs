use std::sync::Arc;

use rcc_macros::ParseFrames;

use crate::{Db, Frame};
/// https://redis.io/commands/zrank
#[derive(Debug, ParseFrames)]
pub struct Zrank {
    pub key: Arc<[u8]>,
    pub member: String,
}

impl<'a> From<&'a Zrank> for dict::cmd::sorted_set::rank::Req<'a> {
    fn from(old: &'a Zrank) -> Self {
        Self {
            key: &old.key,
            member: &old.member,
            rev: false,
        }
    }
}

impl Zrank {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = match db.sorted_set_rank((&self).into())? {
            None => Frame::Null,
            Some(v) => Frame::Integer(v as _),
        };
        Ok(response)
    }
}
