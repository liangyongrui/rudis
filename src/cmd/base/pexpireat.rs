use chrono::{DateTime, NaiveDateTime, Utc};
use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{
    db::{data_type::SimpleType, Db},
    Connection, Frame,
};

/// https://redis.io/commands/pexpireat
#[derive(Debug, ParseFrames)]
pub struct Pexpireat {
    pub key: SimpleType,
    pub ms_timestamp: u64,
}
impl Pexpireat {
    #[instrument(skip(self, db, dst))]
    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
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
        dst.write_frame(&response).await?;

        Ok(())
    }
}
