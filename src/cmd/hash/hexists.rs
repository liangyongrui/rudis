use tracing::{debug, instrument};

use crate::{Connection, Db, Frame, Parse};

/// https://redis.io/commands/hexists
#[derive(Debug)]
pub struct Hexists {
    key: String,
    field: String,
}

impl Hexists {
    pub(crate) fn parse_frames(parse: &mut Parse) -> crate::Result<Self> {
        let key = parse.next_string()?;
        let field = parse.next_string()?;
        Ok(Self { key, field })
    }

    #[instrument(skip(self, db, dst))]
    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let response = match db.hexists(&self.key, self.field) {
            Ok(i) => Frame::Integer(i as _),
            Err(e) => Frame::Error(e),
        };
        debug!(?response);
        // Write the response back to the client
        dst.write_frame(&response).await?;
        Ok(())
    }
}
