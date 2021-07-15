use rcc_macros::ParseFrames;
use tracing::{debug, instrument};

use crate::{db::data_type::SimpleType, Connection, Db, Frame};

/// https://redis.io/commands/exists
#[derive(Debug, ParseFrames)]
pub struct Exists {
    pub keys: Vec<SimpleType>,
}

impl Exists {
    #[instrument(skip(self, db, dst))]
    pub async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let response = Frame::Integer(db.exists(self.keys) as i64);
        debug!(?response);
        // Write the response back to the client
        dst.write_frame(&response).await?;
        Ok(())
    }
}
