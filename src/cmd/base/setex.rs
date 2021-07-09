use bytes::Bytes;
use chrono::{Duration, Utc};
use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{
    db::{data_type::SimpleType, Db},
    utils::options::NxXx,
    Connection, Frame,
};

#[derive(Debug, ParseFrames)]
pub struct Setex {
    key: SimpleType,
    seconds: u64,
    value: Bytes,
}
impl Setex {
    #[instrument(skip(self, db, dst))]
    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let response = if let Err(e) = db
            .set(
                self.key,
                self.value.into(),
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
