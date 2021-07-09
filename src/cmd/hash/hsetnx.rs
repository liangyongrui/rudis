use rcc_macros::ParseFrames;
use tracing::{debug, instrument};

use crate::{db::data_type::SimpleType, Connection, Db, Frame, Parse};

/// https://redis.io/commands/hsetnx
#[derive(Debug, ParseFrames)]
pub struct Hsetnx {
    key: SimpleType,
    field: SimpleType,
    value: SimpleType,
}

impl Hsetnx {
    #[instrument(skip(self, db, dst))]
    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let response = match db.hsetnx(&self.key, self.field, self.value) {
            Ok(i) => Frame::Integer(i as _),
            Err(e) => Frame::Error(e),
        };
        debug!(?response);
        // Write the response back to the client
        dst.write_frame(&response).await?;
        Ok(())
    }
}
