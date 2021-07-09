use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{db::Db, Connection, Frame};

/// https://redis.io/commands/decrby
#[derive(Debug, ParseFrames)]
pub struct Decrby {
    key: String,
    value: i64,
}
impl Decrby {
    #[instrument(skip(self, db, dst))]
    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let response = match db.incr_by(self.key, -self.value) {
            Ok(i) => Frame::Integer(i),
            Err(e) => Frame::Error(e),
        };
        dst.write_frame(&response).await?;

        Ok(())
    }
}
