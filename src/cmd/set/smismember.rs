use rcc_macros::ParseFrames;
use tracing::{debug, instrument};

use crate::{db::data_type::SimpleType, Connection, Db, Frame};
/// https://redis.io/commands/smismember
#[derive(Debug, ParseFrames)]
pub struct Smismember {
    key: SimpleType,
    values: Vec<SimpleType>,
}

impl Smismember {
    #[instrument(skip(self, db, dst))]
    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let response = match db.smismember(&self.key, self.values.iter().collect()) {
            Ok(i) => Frame::Array(
                i.into_iter()
                    .map(|t| if t { 1 } else { 0 })
                    .map(Frame::Integer)
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
