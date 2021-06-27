use chrono::{Duration, Utc};
use tracing::instrument;

use crate::{db::Db, parse::Parse, Connection, Frame};

/// https://redis.io/commands/pexpire
#[derive(Debug)]
pub struct Pexpire {
    key: String,
    milliseconds: u64,
}
impl Pexpire {
    pub(crate) fn parse_frames(parse: &mut Parse) -> crate::Result<Self> {
        let key = parse.next_string()?;
        let milliseconds = parse.next_int()?;
        Ok(Self {
            key,
            milliseconds: milliseconds as u64,
        })
    }

    #[instrument(skip(self, db, dst))]
    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let res = if let Some(ea) =
            Utc::now().checked_add_signed(Duration::milliseconds(self.milliseconds as _))
        {
            db.expires_at(self.key, ea)
        } else {
            false
        };
        // Create a success response and write it to `dst`.
        let response = Frame::Integer(if res { 1 } else { 0 });
        dst.write_frame(&response).await?;

        Ok(())
    }
}
