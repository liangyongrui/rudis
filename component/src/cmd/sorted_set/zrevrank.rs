use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{slot::data_type::SimpleType, Db, Frame};
/// https://redis.io/commands/zrevrank
#[derive(Debug, ParseFrames)]
pub struct Zrevrank {
    pub key: Vec<u8>,
    pub member: SimpleType,
}
impl<'a> From<&'a Zrevrank> for crate::slot::cmd::sorted_set::rank::Req<'a> {
    fn from(old: &'a Zrevrank) -> Self {
        Self {
            key: &old.key,
            member: &old.member,
            rev: true,
        }
    }
}

impl Zrevrank {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = match db.sorted_set_rank((&self).into())? {
            None => Frame::Null,
            Some(v) => Frame::Integer(v as _),
        };
        Ok(response)
    }
}
