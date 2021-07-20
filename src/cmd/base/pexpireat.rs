use chrono::{DateTime, NaiveDateTime, Utc};
use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{
    db::{data_type::SimpleType, Db},
    Frame,
};

/// https://redis.io/commands/pexpireat
#[derive(Debug, Clone, ParseFrames)]
pub struct Pexpireat {
    pub key: SimpleType,
    pub ms_timestamp: u64,
}
impl Pexpireat {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let res = db
            .expires_at(
                &self.key,
                DateTime::<Utc>::from_utc(
                    NaiveDateTime::from_timestamp(self.ms_timestamp as i64 / 1000, 0),
                    Utc,
                ),
            )
            .await;
        // Create a success response and write it to `dst`.
        let response = Frame::Integer(if res { 1 } else { 0 });
        Ok(response)
    }
}
