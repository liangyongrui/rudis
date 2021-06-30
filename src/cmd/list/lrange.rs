use tracing::{debug, instrument};

use crate::{Connection, Db, Frame, Parse};

/// https://redis.io/commands/lrange
#[derive(Debug)]
pub struct Lrange {
    key: String,
    start: i64,
    stop: i64,
}

impl Lrange {
    pub(crate) fn parse_frames(parse: &mut Parse) -> crate::Result<Self> {
        let key = parse.next_string()?;
        let start = parse.next_int()?;
        let stop = parse.next_int()?;
        Ok(Self { key, start, stop })
    }

    #[instrument(skip(self, db, dst))]
    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let response = match db.lrange(&self.key, self.start, self.stop) {
            Ok(r) => Frame::Array(r.into_iter().map(|t| Frame::Bulk(t.get_inner())).collect()),
            Err(e) => Frame::Error(e),
        };
        debug!(?response);
        // Write the response back to the client
        dst.write_frame(&response).await?;
        Ok(())
    }
}