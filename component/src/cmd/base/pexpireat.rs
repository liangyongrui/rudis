use chrono::{DateTime, NaiveDateTime, Utc};
use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{db::Db, slot::data_type::KeyType, Frame};

/// https://redis.io/commands/pexpireat
#[derive(Debug, Clone, ParseFrames)]
pub struct Pexpireat {
    pub key: KeyType,
    pub ms_timestamp: u64,
}
impl From<Pexpireat> for crate::slot::cmd::simple::expire::Req {
    fn from(old: Pexpireat) -> Self {
        // Create a NaiveDateTime from the timestamp
        let naive = NaiveDateTime::from_timestamp(
            (old.ms_timestamp / 1000) as _,
            ((old.ms_timestamp % 1000) as u32) * 1000000,
        );

        // Create a normal DateTime from the NaiveDateTime
        let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
        Self {
            key: old.key,
            expires_at: Some(datetime),
        }
    }
}

impl Pexpireat {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let res = db.expire(self.into()).await?;
        let response = Frame::Integer(if res { 1 } else { 0 });
        Ok(response)
    }
}
