use tracing::{debug, instrument};

use crate::{
    db::Db,
    parse::{Parse, ParseError},
    Connection, Frame,
};

/// https://redis.io/commands/rpop
#[derive(Debug)]
pub struct Rpop {
    key: String,
    count: Option<i64>,
}
impl Rpop {
    pub(crate) fn parse_frames(parse: &mut Parse) -> crate::Result<Self> {
        let key = parse.next_string()?;
        let count = match parse.next_int() {
            Ok(value) => {
                if value <= 0 {
                    return Err("count must be greater than 0.".into());
                }
                Some(value)
            }
            Err(ParseError::EndOfStream) => None,
            Err(err) => return Err(err.into()),
        };
        Ok(Self { key, count })
    }

    #[instrument(skip(self, db, dst))]
    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let response = match db.rpop(&self.key, self.count.unwrap_or(1) as _) {
            Ok(Some(r)) => {
                Frame::Array(r.into_iter().map(|t| Frame::Bulk(t.get_inner())).collect())
            }
            Ok(None) => Frame::Null,
            Err(e) => Frame::Error(e),
        };
        debug!(?response);
        // Write the response back to the client
        dst.write_frame(&response).await?;
        Ok(())
    }
}
