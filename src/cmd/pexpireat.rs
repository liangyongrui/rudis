use chrono::{DateTime, NaiveDateTime, Utc};
use tracing::instrument;

use crate::{db::Db, parse::Parse, Connection, Frame};

/// https://redis.io/commands/pexpireat
#[derive(Debug)]
pub struct Pexpireat {
    key: String,
    ms_timestamp: u64,
}
impl Pexpireat {
    pub(crate) fn parse_frames(parse: &mut Parse) -> crate::Result<Self> {
        let key = parse.next_string()?;
        let ms_timestamp = parse.next_int()?;
        Ok(Self {
            key,
            ms_timestamp: ms_timestamp as u64,
        })
    }

    /// Apply the `Set` command to the specified `Db` instance.
    ///
    /// The response is written to `dst`. This is called by the server in order
    /// to execute a received command.
    #[instrument(skip(self, db, dst))]
    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let res = db.pexpireat(
            self.key,
            DateTime::<Utc>::from_utc(
                NaiveDateTime::from_timestamp(self.ms_timestamp as i64 / 1000, 0),
                Utc,
            ),
        );
        // Create a success response and write it to `dst`.
        let response = Frame::Integer(if res { 1 } else { 0 });
        dst.write_frame(&response).await?;

        Ok(())
    }
}
