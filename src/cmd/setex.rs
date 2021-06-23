use bytes::Bytes;
use chrono::{Duration, Utc};
use tracing::instrument;

use crate::{db::Db, parse::Parse, Connection, Frame};

#[derive(Debug)]
pub struct Setex {
    /// Name of the key to get
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

    /// Apply the `Set` command to the specified `Db` instance.
    ///
    /// The response is written to `dst`. This is called by the server in order
    /// to execute a received command.
    #[instrument(skip(self, db, dst))]
    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        db.set(
            self.key,
            self.value,
            None,
            Utc::now().checked_add_signed(Duration::seconds(self.seconds as i64)),
            false,
        );
        // Create a success response and write it to `dst`.
        let response = Frame::Simple("OK".to_string());
        dst.write_frame(&response).await?;

        Ok(())
    }
}