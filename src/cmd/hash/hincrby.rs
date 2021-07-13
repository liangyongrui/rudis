use rcc_macros::ParseFrames;
use tracing::{debug, instrument};

use crate::{db::data_type::SimpleType, Connection, Db, Frame};
/// https://redis.io/commands/hincrby
#[derive(Debug, ParseFrames)]
pub struct Hincrby {
    pub key: SimpleType,
    pub field: SimpleType,
    pub value: i64,
}

impl Hincrby {
    #[instrument(skip(self, db, dst))]
    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let response = match db.hincrby(&self.key, self.field, self.value) {
            Ok(i) => Frame::Integer(i),
            Err(e) => Frame::Error(e),
        };
        debug!(?response);
        // Write the response back to the client
        dst.write_frame(&response).await?;
        Ok(())
    }
}
