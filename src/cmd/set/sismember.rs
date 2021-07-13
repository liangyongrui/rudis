use rcc_macros::ParseFrames;
use tracing::{debug, instrument};

use crate::{db::data_type::SimpleType, Connection, Db, Frame};

/// https://redis.io/commands/sismember
#[derive(Debug, ParseFrames)]
pub struct Sismember {
    pub key: SimpleType,
    pub value: SimpleType,
}

impl Sismember {
    #[instrument(skip(self, db, dst))]
    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let response = match db.sismember(&self.key, &self.value) {
            Ok(i) => Frame::Integer(if i { 1 } else { 0 }),
            Err(e) => Frame::Error(e),
        };
        debug!(?response);
        // Write the response back to the client
        dst.write_frame(&response).await?;
        Ok(())
    }
}
