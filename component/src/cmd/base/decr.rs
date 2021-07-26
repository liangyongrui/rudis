use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{
    db::Db,
    slot::{self, data_type::KeyType},
    Frame,
};

/// https://redis.io/commands/decr
#[derive(Debug, Clone, ParseFrames)]
pub struct Decr {
    pub key: KeyType,
}

impl From<Decr> for slot::cmd::simple::incr::Req {
    fn from(decr: Decr) -> Self {
        Self {
            key: decr.key,
            value: -1,
        }
    }
}

impl Decr {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = db.incr(self.into()).await?;
        Ok(Frame::Integer(response))
    }
}
