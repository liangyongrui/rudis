use bytes::Bytes;
use chrono::{Duration, Utc};
use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{db::Db, utils::options::NxXx, Connection, Frame};

#[derive(Debug, ParseFrames)]
pub struct Psetex {
    /// Name of the key to get
    key: String,
    milliseconds: u64,
    value: Bytes,
}
impl Psetex {
    #[instrument(skip(self, db, dst))]
    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let response = if let Err(e) = db
            .set(
                self.key,
                self.value.into(),
                NxXx::None,
                Utc::now().checked_add_signed(Duration::milliseconds(self.milliseconds as i64)),
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
