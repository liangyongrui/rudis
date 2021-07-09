use rcc_macros::ParseFrames;
use tracing::{debug, instrument};

use crate::{Connection, Db, Frame};
/// https://redis.io/commands/del
#[derive(Debug, ParseFrames)]
pub struct Del {
    keys: Vec<String>,
}

impl Del {
    #[instrument(skip(self, db, dst))]
    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let response = Frame::Integer(db.del(self.keys) as i64);
        debug!(?response);
        // Write the response back to the client
        dst.write_frame(&response).await?;
        Ok(())
    }
}
