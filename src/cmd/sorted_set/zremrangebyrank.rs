use rcc_macros::ParseFrames;
use tracing::{debug, instrument};

use crate::{db::data_type::SimpleType, Connection, Db, Frame};

/// https://redis.io/commands/zremrangebyrank
#[derive(Debug, Clone, ParseFrames)]
pub struct Zremrangebyrank {
    pub key: SimpleType,
    pub range: (i64, i64),
}

impl Zremrangebyrank {
    #[instrument(skip(self, db, dst))]
    pub async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let response = match db.zremrange_by_rank(&self.key, self.range) {
            Ok(v) => Frame::Integer(v as _),
            Err(e) => Frame::Error(e),
        };
        debug!(?response);
        // Write the response back to the client
        dst.write_frame(&response).await?;
        Ok(())
    }
}
