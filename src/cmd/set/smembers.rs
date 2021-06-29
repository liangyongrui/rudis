use tracing::{debug, instrument};

use crate::{Connection, Db, Frame, Parse};

/// https://redis.io/commands/smembers
#[derive(Debug)]
pub struct Smembers {
    key: String,
}

impl Smembers {
    pub(crate) fn parse_frames(parse: &mut Parse) -> crate::Result<Self> {
        let key = parse.next_string()?;
        Ok(Self { key })
    }

    #[instrument(skip(self, db, dst))]
    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let response = match db.smembers(&self.key) {
            Ok(i) => Frame::Array(i.iter().map(|t| t.clone().into()).collect()),
            Err(e) => Frame::Error(e),
        };
        debug!(?response);
        // Write the response back to the client
        dst.write_frame(&response).await?;
        Ok(())
    }
}
