use bytes::Bytes;
use chrono::{Duration, Utc};
use tracing::instrument;

use crate::{db::Db, options::NxXx, parse::Parse, Connection, Frame};

#[derive(Debug)]
pub struct Setex {
    key: String,
    seconds: u64,
    value: Bytes,
}
impl Setex {
    pub(crate) fn parse_frames(parse: &mut Parse) -> crate::Result<Self> {
        let key = parse.next_string()?;
        let milliseconds = parse.next_int()?;
        let value = parse.next_bytes()?;
        Ok(Self {
            key,
            seconds: milliseconds as u64,
            value,
        })
    }

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