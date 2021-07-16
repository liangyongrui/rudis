use rcc_macros::ParseFrames;
use tracing::{debug, instrument};

use crate::{
    db::{data_type::SimpleType, Db},
    parse::{Parse, ParseError},
    Connection, Frame,
};
/// https://redis.io/commands/lpop
#[derive(Debug, Clone, ParseFrames)]
pub struct Lpop {
    pub key: SimpleType,
    pub count: Option<i64>,
}
impl Lpop {
    #[instrument(skip(self, db, dst))]
    pub async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let response = match db.lpop(&self.key, self.count.unwrap_or(1) as _) {
            Ok(Some(r)) => Frame::Array(r.into_iter().map(|t| t.into()).collect()),
            Ok(None) => Frame::Null,
            Err(e) => Frame::Error(e),
        };
        debug!(?response);
        // Write the response back to the client
        dst.write_frame(&response).await?;
        Ok(())
    }
}
