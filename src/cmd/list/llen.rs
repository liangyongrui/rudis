use tracing::{debug, instrument};

use crate::{db::Db, parse::Parse, Connection, Frame};

/// https://redis.io/commands/llen
#[derive(Debug)]
pub struct Llen {
    key: String,
}
impl Llen {
    pub(crate) fn parse_frames(parse: &mut Parse) -> crate::Result<Self> {
        let key = parse.next_string()?;
        Ok(Self { key })
    }

    #[instrument(skip(self, db, dst))]
    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let response = match db.llen(&self.key) {
            Ok(r) => Frame::Integer(r as _),
            Err(e) => Frame::Error(e),
        };
        debug!(?response);
        // Write the response back to the client
        dst.write_frame(&response).await?;
        Ok(())
    }
}
