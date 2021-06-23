use tracing::instrument;

use crate::{db::Db, parse::Parse, Connection, Frame};

#[derive(Debug)]
pub struct Incrby {
    key: String,
    value: i64,
}
impl Incrby {
    pub(crate) fn parse_frames(parse: &mut Parse) -> crate::Result<Self> {
        let key = parse.next_string()?;
        let value = parse.next_int()?;
        Ok(Self { key, value })
    }

    #[instrument(skip(self, db, dst))]
    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let response = match db.incr_by(self.key, self.value) {
            Ok(i) => Frame::Integer(i),
            Err(e) => Frame::Error(e),
        };
        dst.write_frame(&response).await?;

        Ok(())
    }
}
