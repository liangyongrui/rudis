use bytes::Bytes;
use tracing::{debug, instrument};

use crate::{parse::ParseError, Connection, Db, Frame, Parse};

/// https://redis.io/commands/exists
#[derive(Debug)]
pub struct Exists {
    keys: Vec<String>,
}

impl Exists {
    pub(crate) fn parse_frames(parse: &mut Parse) -> crate::Result<Self> {
        let mut keys = vec![];
        loop {
            match parse.next_string() {
                Ok(key) => keys.push(key),
                Err(ParseError::EndOfStream) => break,
                Err(err) => return Err(err.into()),
            }
        }
        Ok(Self { keys })
    }

    #[instrument(skip(self, db, dst))]
    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let response = Frame::Integer(db.exists(self.keys) as i64);
        debug!(?response);
        // Write the response back to the client
        dst.write_frame(&response).await?;
        Ok(())
    }

    pub(crate) fn _into_frame(self) -> Frame {
        let mut frame = Frame::array();
        frame.push_bulk(Bytes::from("exists".as_bytes()));
        for key in self.keys {
            frame.push_bulk(Bytes::from(key.into_bytes()));
        }
        frame
    }
}
