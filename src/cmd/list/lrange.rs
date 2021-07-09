use rcc_macros::ParseFrames;
use tracing::{debug, instrument};

use crate::{db::data_type::SimpleType, Connection, Db, Frame};

/// https://redis.io/commands/lrange
#[derive(Debug, ParseFrames)]
pub struct Lrange {
    key: SimpleType,
    start: i64,
    stop: i64,
}

impl Lrange {
    #[instrument(skip(self, db, dst))]
    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let response = match db.lrange(&self.key, self.start, self.stop) {
            Ok(r) => Frame::Array(r.into_iter().map(|t| t.into()).collect()),
            Err(e) => Frame::Error(e),
        };
        debug!(?response);
        // Write the response back to the client
        dst.write_frame(&response).await?;
        Ok(())
    }
}
