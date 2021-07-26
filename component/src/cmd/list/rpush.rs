use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{
    db::Db,
    slot::data_type::{KeyType, SimpleType},
    Frame,
};

/// https://redis.io/commands/rpush
#[derive(Debug, Clone, ParseFrames)]
pub struct Rpush {
    pub key: KeyType,
    pub values: Vec<SimpleType>,
}

impl From<Rpush> for crate::slot::cmd::deque::push::Req {
    fn from(old: Rpush) -> Self {
        Self {
            key: old.key,
            left: false,
            elements: old.values,
            nx_xx: crate::utils::options::NxXx::None,
        }
    }
}

impl Rpush {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = db.deque_push(self.into()).await?;
        Ok(Frame::Integer(response.new_len as _))
    }
}
