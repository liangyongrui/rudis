use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{
    db::{data_type::SimpleType, Db},
    Connection, Frame,
};

/// https://redis.io/commands/incrby
#[derive(Debug, Clone, ParseFrames)]
pub struct Incrby {
    pub key: SimpleType,
    pub value: i64,
}
impl Incrby {
    #[instrument(skip(self, db, dst))]
    pub async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let response = match db.incr_by(self.key, self.value) {
            Ok(i) => Frame::Integer(i),
            Err(e) => Frame::Error(e),
        };
        dst.write_frame(&response).await?;

        Ok(())
    }
}
