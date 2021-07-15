use rcc_macros::ParseFrames;
use tracing::{debug, instrument};

use crate::{db::data_type::SimpleType, Connection, Db, Frame};
/// https://redis.io/commands/hgetall
#[derive(Debug, ParseFrames)]
pub struct Hgetall {
    pub key: SimpleType,
}

impl Hgetall {
    #[instrument(skip(self, db, dst))]
    pub async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let response = match db.hgetall(&self.key) {
            Ok(v) => Frame::Array(
                v.into_iter()
                    .flat_map(|i| vec![i.field.into(), i.value.into()].into_iter())
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
