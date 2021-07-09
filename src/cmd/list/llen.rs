use rcc_macros::ParseFrames;
use tracing::{debug, instrument};

use crate::{
    db::{data_type::SimpleType, Db},
    Connection, Frame,
};

/// https://redis.io/commands/llen
#[derive(Debug, ParseFrames)]
pub struct Llen {
    key: SimpleType,
}
impl Llen {
    #[instrument(skip(self, db, dst))]
    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let response = match db.llen(&self.key) {
            Ok(r) => Frame::Integer(r as _),
            Err(e) => Frame::Error(e),
        };
        debug!(?response);
        // Write the response back to the client
        dst.write_frame(&response).await?;
        Ok(())
    }
}
