use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{slot::data_type::SimpleType, Db, Frame};

/// https://redis.io/commands/sismember
#[derive(Debug, ParseFrames)]
pub struct Sismember {
    pub key: Vec<u8>,
    pub value: SimpleType,
}

impl<'a> From<&'a Sismember> for crate::slot::cmd::set::exists::Req<'a> {
    fn from(old: &'a Sismember) -> Self {
        Self {
            key: &old.key,
            field: &old.value,
        }
    }
}

impl Sismember {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let res = db.set_exists((&self).into())?;
        Ok(Frame::Integer(if res { 1 } else { 0 }))
    }
}
