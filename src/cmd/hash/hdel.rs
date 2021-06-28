use tracing::{debug, instrument};

use crate::{parse::ParseError, Connection, Db, Frame, Parse};

/// https://redis.io/commands/hdel
#[derive(Debug)]
pub struct Hdel {
    key: String,
    fields: Vec<String>,
}

impl Hdel {
    pub(crate) fn parse_frames(parse: &mut Parse) -> crate::Result<Self> {
        let key = parse.next_string()?;
        let mut fields = vec![parse.next_string()?];
        loop {
            match parse.next_string() {
                Ok(s) => fields.push(s),
                Err(ParseError::EndOfStream) => break,
                Err(err) => return Err(err.into()),
            }
        }
        Ok(Self { key, fields })
    }

    #[instrument(skip(self, db, dst))]
    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let response = match db.hdel(&self.key, self.fields) {
            Ok(i) => Frame::Integer(i as _),
            Err(e) => Frame::Error(e),
        };
        debug!(?response);
        // Write the response back to the client
        dst.write_frame(&response).await?;
        Ok(())
    }
}
