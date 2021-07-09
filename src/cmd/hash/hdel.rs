use rcc_macros::ParseFrames;
use tracing::{debug, instrument};

use crate::{Connection, Db, Frame};

/// https://redis.io/commands/hdel
#[derive(Debug, ParseFrames)]
pub struct Hdel {
    key: String,
    fields: Vec<String>,
}

impl Hdel {
    #[instrument(skip(self, db, dst))]
    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let response = match db.hdel(&self.key, self.fields) {
            Ok(i) => Frame::Integer(i as _),
            Err(e) => Frame::Error(e),
        };
        debug!(?response);
        // Write the response back to the client
        dst.write_frame(&response).await?;
        Ok(())
    }
}
