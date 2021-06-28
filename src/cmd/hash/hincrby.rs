use tracing::{debug, instrument};

use crate::{Connection, Db, Frame, Parse};

/// https://redis.io/commands/hincrby
#[derive(Debug)]
pub struct Hincrby {
    key: String,
    field: String,
    value: i64,
}

impl Hincrby {
    pub(crate) fn parse_frames(parse: &mut Parse) -> crate::Result<Self> {
        let key = parse.next_string()?;
        let field = parse.next_string()?;
        let value = parse.next_int()?;
        Ok(Self { key, field, value })
    }

    #[instrument(skip(self, db, dst))]
    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let response = match db.hincrby(&self.key, self.field, self.value) {
            Ok(i) => Frame::Integer(i),
            Err(e) => Frame::Error(e),
        };
        debug!(?response);
        // Write the response back to the client
        dst.write_frame(&response).await?;
        Ok(())
    }
}
