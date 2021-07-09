use bytes::Bytes;
use tracing::{debug, instrument};

use crate::{parse::ParseError, Connection, Db, Frame, Parse};

/// https://redis.io/commands/rpushx
#[derive(Debug)]
pub struct Rpushx {
    key: String,
    values: Vec<Bytes>,
}

impl Rpushx {
    pub(crate) fn parse_frames(parse: &mut Parse) -> crate::Result<Self> {
        let key = parse.next_string()?;
        let mut values = vec![parse.next_bytes()?];
        loop {
            match parse.next_bytes() {
                Ok(value) => values.push(value),
                Err(ParseError::EndOfStream) => break,
                Err(err) => return Err(err.into()),
            }
        }
        Ok(Self { key, values })
    }

    #[instrument(skip(self, db, dst))]
    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let response = match db.rpushx(&self.key, self.values) {
            Ok(i) => Frame::Integer(i as _),
            Err(e) => Frame::Error(e),
        };
        debug!(?response);
        // Write the response back to the client
        dst.write_frame(&response).await?;
        Ok(())
    }
}
