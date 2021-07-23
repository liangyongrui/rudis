use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{db::Db, slot::data_type::SimpleType, Frame};

/// https://redis.io/commands/hget
#[derive(Debug, ParseFrames)]
pub struct Hget {
    pub key: SimpleType,
    pub field: SimpleType,
}

impl<'a> From<&'a Hget> for crate::slot::cmd::kvp::get::Req<'a> {
    fn from(old: &'a Hget) -> Self {
        Self {
            key: &old.key,
            field: &old.field,
        }
    }
}

impl Hget {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        Ok((&db.kvp_get((&self).into())?).into())
    }
}
