use rcc_macros::ParseFrames;
use tracing::{debug, instrument};

use crate::{db::data_type::SimpleType, Connection, Db, Frame};

/// https://redis.io/commands/smembers
#[derive(Debug, ParseFrames)]
pub struct Smembers {
    pub key: SimpleType,
}

impl Smembers {
    #[instrument(skip(self, db, dst))]
    pub async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let response = match db.smembers(&self.key) {
            Ok(i) => Frame::Array(i.iter().map(|t| t.clone().into()).collect()),
            Err(e) => Frame::Error(e),
        };
        debug!(?response);
        // Write the response back to the client
        dst.write_frame(&response).await?;
        Ok(())
    }
}
