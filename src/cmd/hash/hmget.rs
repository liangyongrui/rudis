use rcc_macros::ParseFrames;
use tracing::{debug, instrument};

use crate::{db::data_type::SimpleType, Connection, Db, Frame};
/// https://redis.io/commands/hmget
#[derive(Debug, ParseFrames)]
pub struct Hmget {
    pub key: SimpleType,
    pub fields: Vec<SimpleType>,
}

impl Hmget {
    #[instrument(skip(self, db, dst))]
    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let response = match db.hmget(&self.key, self.fields) {
            Ok(v) => Frame::Array(
                v.into_iter()
                    .map(|x| x.map(|y| y.into()).unwrap_or(Frame::Null))
                    .collect(),
            ),
            Err(e) => Frame::Error(e),
        };
        debug!(?response);
        // Write the response back to the client
        dst.write_frame(&response).await?;
        Ok(())
    }
}
