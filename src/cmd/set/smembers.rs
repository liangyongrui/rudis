use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{slot::data_type::SimpleType, Db, Frame};

/// https://redis.io/commands/smembers
#[derive(Debug, ParseFrames)]
pub struct Smembers {
    pub key: SimpleType,
}

impl From<Smembers> for crate::slot::cmd::set::get_all::Req<'_> {
    fn from(old: Smembers) -> Self {
        Self { key: &old.key }
    }
}

impl Smembers {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        if let Some(res) = db.set_get_all(self.into())? {
            Ok(Frame::Array(res.iter().map(|t| t.into()).collect()))
        } else {
            Ok(Frame::Array(vec![]))
        }
    }
}
