use bytes::Bytes;
use rcc_macros::ParseFrames;
use tracing::{debug, instrument};

use crate::{Connection, Db, Frame};

/// https://redis.io/commands/exists
#[derive(Debug, ParseFrames)]
pub struct Exists {
    keys: Vec<String>,
}

impl Exists {
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
