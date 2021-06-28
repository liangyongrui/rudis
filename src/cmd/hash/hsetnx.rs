use tracing::{debug, instrument};

use crate::{db::data_type::SimpleType, Connection, Db, Frame, Parse};

/// https://redis.io/commands/hsetnx
#[derive(Debug)]
pub struct Hsetnx {
    key: String,
    field: String,
    value: SimpleType,
}

impl Hsetnx {
    pub(crate) fn parse_frames(parse: &mut Parse) -> crate::Result<Self> {
        let key = parse.next_string()?;
        let field = parse.next_string()?;
        let value = match parse.next()? {
            Frame::Simple(s) => SimpleType::SimpleString(s),
            Frame::Bulk(data) => data.into(),
            Frame::Integer(data) => data.into(),
            frame => {
                return Err(format!(
                    "protocol error; expected simple frame or bulk frame, got {:?}",
                    frame
                )
                .into())
            }
        };
        Ok(Self { key, field, value })
    }

    #[instrument(skip(self, db, dst))]
    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let response = match db.hsetnx(&self.key, self.field, self.value) {
            Ok(i) => Frame::Integer(i as _),
            Err(e) => Frame::Error(e),
        };
        debug!(?response);
        // Write the response back to the client
        dst.write_frame(&response).await?;
        Ok(())
    }
}
