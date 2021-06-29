use tracing::instrument;

use crate::{db::Db, parse::Parse, Connection, Frame};

/// https://redis.io/commands/incr
#[derive(Debug)]
pub struct Incr {
    key: String,
}
impl Incr {
    pub(crate) fn parse_frames(parse: &mut Parse) -> crate::Result<Self> {
        let key = parse.next_string()?;
        Ok(Self { key })
    }

    #[instrument(skip(self, db, dst))]
    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let response = match db.incr_by(self.key, 1) {
            Ok(i) => Frame::Integer(i),
            Err(e) => Frame::Error(e),
        };
        dst.write_frame(&response).await?;

        Ok(())
    }
}
