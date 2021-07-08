use tracing::{debug, instrument};

use crate::{Connection, Db, Frame, Parse};

/// https://redis.io/commands/zrank
#[derive(Debug)]
pub struct Zrank {
    key: String,
    member: String,
}

impl Zrank {

    pub(crate) fn parse_frames(parse: &mut Parse) -> crate::Result<Self> {
        let key = parse.next_string()?;
        let member = parse.next_string()?;
        Ok(Self { key, member })
    }

    #[instrument(skip(self, db, dst))]
    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let response = match db.zrank(&self.key, &self.member, false) {
            Ok(None) => Frame::Null,
            Ok(Some(v)) => Frame::Integer(v as _),
            Err(e) => Frame::Error(e),
        };
        debug!(?response);
        // Write the response back to the client
        dst.write_frame(&response).await?;
        Ok(())
    }
}
