use chrono::{DateTime, NaiveDateTime, Utc};
use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{
    db::{data_type::SimpleType, Db},
    Connection, Frame,
};

/// https://redis.io/commands/expireat
#[derive(Debug, Clone, ParseFrames)]
pub struct Expireat {
    pub key: SimpleType,
    pub s_timestamp: u64,
}
impl Expireat {
    /// Apply the `Set` command to the specified `Db` instance.
    ///
    /// The response is written to `dst`. This is called by the server in order
    /// to execute a received command.
    #[instrument(skip(self, db, dst))]
    pub async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let res = db
            .expires_at(
                &self.key,
                DateTime::<Utc>::from_utc(
                    NaiveDateTime::from_timestamp(self.s_timestamp as i64, 0),
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
