use chrono::{DateTime, NaiveDateTime, Utc};
use tracing::instrument;

use crate::{db::Db, parse::Parse, Connection, Frame};

/// https://redis.io/commands/expireat
#[derive(Debug)]
pub struct Expireat {
    key: String,
    s_timestamp: u64,
}
impl Expireat {
    pub(crate) fn parse_frames(parse: &mut Parse) -> crate::Result<Self> {
        let key = parse.next_string()?;
        let s_timestamp = parse.next_int()?;
        Ok(Self {
            key,
            s_timestamp: s_timestamp as u64,
        })
    }

    /// Apply the `Set` command to the specified `Db` instance.
    ///
    /// The response is written to `dst`. This is called by the server in order
    /// to execute a received command.
    #[instrument(skip(self, db, dst))]
    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let res = db.expires_at(
            self.key,
            DateTime::<Utc>::from_utc(
                NaiveDateTime::from_timestamp(self.s_timestamp as i64, 0),
                Utc,
            ),
        );
        // Create a success response and write it to `dst`.
        let response = Frame::Integer(if res { 1 } else { 0 });
        dst.write_frame(&response).await?;

        Ok(())
    }
}
