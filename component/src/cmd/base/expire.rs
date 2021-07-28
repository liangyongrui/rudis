use chrono::{Duration, Utc};
use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{db::Db, Frame};

/// https://redis.io/commands/expire
#[derive(Debug, Clone, ParseFrames)]
pub struct Expire {
    pub key: Vec<u8>,
    pub seconds: u64,
}

impl From<Expire> for crate::slot::cmd::simple::expire::Req {
    fn from(old: Expire) -> Self {
        Self {
            key: old.key,
            expires_at: Utc::now().checked_add_signed(Duration::seconds(old.seconds as _)),
        }
    }
}

impl Expire {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let res = db.expire(self.into()).await?;
        let response = Frame::Integer(if res { 1 } else { 0 });
        Ok(response)
    }
}
