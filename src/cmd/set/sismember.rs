use tracing::{debug, instrument};

use crate::{db::data_type::SimpleType, Connection, Db, Frame, Parse};

/// https://redis.io/commands/sismember
#[derive(Debug)]
pub struct Sismember {
    key: String,
    value: SimpleType,
}

impl Sismember {
    pub(crate) fn parse_frames(parse: &mut Parse) -> crate::Result<Self> {
        let key = parse.next_string()?;
        let value = match parse.next() {
            Ok(Frame::Simple(s)) => SimpleType::SimpleString(s),
            Ok(Frame::Bulk(data)) => data.into(),
            Ok(Frame::Integer(data)) => data.into(),
            Ok(frame) => {
                return Err(format!(
                    "protocol error; expected simple frame or bulk frame, got {:?}",
                    frame
                )
                .into())
            }
            Err(err) => return Err(err.into()),
        };
        Ok(Self { key, value })
    }

    #[instrument(skip(self, db, dst))]
    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let response = match db.sismember(&self.key, &self.value) {
            Ok(i) => Frame::Integer(if i { 1 } else { 0 }),
            Err(e) => Frame::Error(e),
        };
        debug!(?response);
        // Write the response back to the client
        dst.write_frame(&response).await?;
        Ok(())
    }
}
