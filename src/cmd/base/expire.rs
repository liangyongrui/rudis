use chrono::{Duration, Utc};
use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{
    db::{data_type::SimpleType, Db},
    Connection, Frame,
};

/// https://redis.io/commands/expire
#[derive(Debug, ParseFrames)]
pub struct Expire {
    pub key: SimpleType,
    pub seconds: u64,
}
impl Expire {
    #[instrument(skip(self, db, dst))]
    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let res =
            if let Some(ea) = Utc::now().checked_add_signed(Duration::seconds(self.seconds as _)) {
                db.expires_at(&self.key, ea).await
            } else {
                false
            };
        // Create a success response and write it to `dst`.
        let response = Frame::Integer(if res { 1 } else { 0 });
        dst.write_frame(&response).await?;

        Ok(())
    }
}
