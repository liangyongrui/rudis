use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{
    slot::data_type::{KeyType, SimpleType},
    Db, Frame,
};

/// https://redis.io/commands/srem
#[derive(Debug, ParseFrames, Clone)]
pub struct Srem {
    pub key: KeyType,
    pub values: Vec<SimpleType>,
}

impl From<Srem> for crate::slot::cmd::set::remove::Req {
    fn from(old: Srem) -> Self {
        Self {
            key: old.key,
            members: old.values,
        }
    }
}
impl Srem {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let res = db.set_remove(self.into()).await?;
        Ok(Frame::Integer((res.old_len - res.new_len) as _))
    }
}
