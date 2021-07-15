use rcc_macros::ParseFrames;
use tracing::{debug, instrument};

use crate::{db::data_type::SimpleType, Connection, Db, Frame};
/// https://redis.io/commands/zrank
#[derive(Debug, ParseFrames)]
pub struct Zrank {
    pub key: SimpleType,
    pub member: SimpleType,
}

impl Zrank {
    #[instrument(skip(self, db, dst))]
    pub async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let response = match db.zrank(&self.key, &self.member, false) {
            Ok(None) => Frame::Null,
            Ok(Some(v)) => Frame::Integer(v as _),
            Err(e) => Frame::Error(e),
        };
        debug!(?response);
        // Write the response back to the client
        dst.write_frame(&response).await?;
        Ok(())
    }
}
