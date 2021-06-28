use tracing::{debug, instrument};

use crate::{Connection, Db, Frame, Parse};

/// https://redis.io/commands/hget
#[derive(Debug)]
pub struct Hget {
    key: String,
    field: String,
}

impl Hget {
    pub(crate) fn parse_frames(parse: &mut Parse) -> crate::Result<Self> {
        let key = parse.next_string()?;
        let field = parse.next_string()?;
        Ok(Hget { key, field })
    }

    #[instrument(skip(self, db, dst))]
    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let response = match db.hget(&self.key, self.field) {
            Ok(Some(v)) => v.into(),
            Ok(None) => Frame::Null,
            Err(e) => Frame::Error(e),
        };
        debug!(?response);
        // Write the response back to the client
        dst.write_frame(&response).await?;
        Ok(())
    }
}
