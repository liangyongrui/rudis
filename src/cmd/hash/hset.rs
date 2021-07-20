use rcc_macros::ParseFrames;
use tracing::{debug, instrument};

use crate::{
    db::data_type::{SimpleType, SimpleTypePair},
    Connection, Db, Frame,
};
/// https://redis.io/commands/hset
#[derive(Debug, Clone, ParseFrames)]
pub struct Hset {
    pub key: SimpleType,
    pub pairs: Vec<SimpleTypePair>,
}

impl Hset {
    #[instrument(skip(self, db, dst))]
    pub async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let response = match db.hset(self.key, self.pairs).await {
            Ok(i) => Frame::Integer(i as _),
            Err(e) => Frame::Error(e),
        };
        debug!(?response);
        // Write the response back to the client
        dst.write_frame(&response).await?;
        Ok(())
    }
}
