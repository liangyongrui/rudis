use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{slot::data_type::SimpleType, Db, Frame};
/// https://redis.io/commands/zrank
#[derive(Debug, ParseFrames)]
pub struct Zrank {
    pub key: Vec<u8>,
    pub member: SimpleType,
}

impl<'a> From<&'a Zrank> for crate::slot::cmd::sorted_set::rank::Req<'a> {
    fn from(old: &'a Zrank) -> Self {
        Self {
            key: &old.key,
            member: &old.member,
            rev: false,
        }
    }
}

impl Zrank {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = match db.sorted_set_rank((&self).into())? {
            None => Frame::Null,
            Some(v) => Frame::Integer(v as _),
        };
        Ok(response)
    }
}
