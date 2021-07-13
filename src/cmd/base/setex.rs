use chrono::{Duration, Utc};
use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{
    db::{data_type::SimpleType, Db},
    utils::options::NxXx,
    Connection, Frame,
};
/// https://redis.io/commands/setex
#[derive(Debug, ParseFrames)]
pub struct Setex {
    pub key: SimpleType,
    pub seconds: u64,
    pub value: SimpleType,
}
impl Setex {
    #[instrument(skip(self, db, dst))]
    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let response = if let Err(e) = db
            .set(
                self.key,
                self.value,
                NxXx::None,
                Utc::now().checked_add_signed(Duration::seconds(self.seconds as i64)),
                false,
            )
            .await
        {
            Frame::Error(e)
        } else {
            Frame::Simple("OK".to_string())
        };
        // Create a success response and write it to `dst`.
        dst.write_frame(&response).await?;

        Ok(())
    }
}
