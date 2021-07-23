use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{db::Db, slot::data_type::SimpleType, Frame};

/// Get the value of key.
///
/// If the key does not exist the special value nil is returned. An error is
/// returned if the value stored at key is not a string, because GET only
/// handles string values.
/// https://redis.io/commands/get
#[derive(Debug, ParseFrames)]
pub struct Get {
    /// Name of the key to get
    pub key: SimpleType,
}
impl<'a> From<&'a Get> for crate::slot::cmd::simple::get::Req<'a> {
    fn from(old: &'a Get) -> Self {
        Self { key: &old.key }
    }
}
impl Get {
    /// Create a new `Get` command which fetches `key`.
    pub fn new(key: impl ToString) -> Get {
        Get {
            key: SimpleType::String(key.to_string().into()),
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
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        // Get the value from the shared database state

        let response = (&db.get((&self).into())?).into();
        // Write the response back to the client
        Ok(response)
    }
}
