use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{
    db::Db,
    slot::data_type::{KeyType, SimpleType},
    Frame,
};

/// https://redis.io/commands/hdel
#[derive(Debug, ParseFrames, Clone)]
pub struct Hdel {
    pub key: KeyType,
    pub fields: Vec<SimpleType>,
}

impl From<Hdel> for crate::slot::cmd::kvp::del::Req {
    fn from(old: Hdel) -> Self {
        Self {
            key: old.key,
            fields: old.fields,
        }
    }
}

impl Hdel {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = db.kvp_del(self.into()).await?;
        Ok(Frame::Integer((response.new_len - response.old_len) as _))
    }
}
