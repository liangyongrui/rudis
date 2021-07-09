use rcc_macros::ParseFrames;
use tracing::{debug, instrument};

use crate::{db::data_type::SimpleType, Connection, Db, Frame};

/// Get the value of key.
///
/// If the key does not exist the special value nil is returned. An error is
/// returned if the value stored at key is not a string, because GET only
/// handles string values.
#[derive(Debug, ParseFrames)]
pub struct Get {
    /// Name of the key to get
    key: SimpleType,
}

impl Get {
    /// Create a new `Get` command which fetches `key`.
    pub fn new(key: impl ToString) -> Get {
        Get {
            key: SimpleType::SimpleString(key.to_string()),
        }
    }

    /// Get the key
    pub fn key(&self) -> &SimpleType {
        &self.key
    }

    /// Apply the `Get` command to the specified `Db` instance.
    ///
    /// The response is written to `dst`. This is called by the server in order
    /// to execute a received command.
    #[instrument(skip(self, db, dst))]
    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        // Get the value from the shared database state

        let response = db
            .get(&self.key)
            .map(|t| t.map(|x| x.into()).unwrap_or_else(|| Frame::Null))
            .unwrap_or_else(Frame::Error);
        debug!(?response);

        // Write the response back to the client
        dst.write_frame(&response).await?;

        Ok(())
    }
}
