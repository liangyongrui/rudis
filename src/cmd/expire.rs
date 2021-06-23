use chrono::{Duration, Utc};
use tracing::instrument;

use crate::{db::Db, parse::Parse, Connection, Frame};

/// https://redis.io/commands/expire
#[derive(Debug)]
pub struct Expire {
    key: String,
    seconds: u64,
}
impl Expire {
    pub(crate) fn parse_frames(parse: &mut Parse) -> crate::Result<Self> {
        let key = parse.next_string()?;
        let seconds = parse.next_int()?;
        Ok(Self {
            key,
            seconds: seconds as u64,
        })
    }

    #[instrument(skip(self, db, dst))]
    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let res =
            if let Some(ea) = Utc::now().checked_add_signed(Duration::seconds(self.seconds as _)) {
                db.expire_at(self.key, ea)
            } else {
                false
            };
        // Create a success response and write it to `dst`.
        let response = Frame::Integer(if res { 1 } else { 0 });
        dst.write_frame(&response).await?;

        Ok(())
    }
}
