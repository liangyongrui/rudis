use db::Db;
use macros::ParseFrames2;

use crate::Frame;

/// Get the value of key.
///
/// If the key does not exist the special value nil is returned. An error is
/// returned if the value stored at key is not a string, because GET only
/// handles string values.
///
/// <https://redis.io/commands/get>
#[derive(Debug, ParseFrames2)]
pub struct Get<'a> {
    /// Name of the key to get
    pub key: &'a [u8],
}

impl Get<'_> {
    /// Apply the `Get` command to the specified `Db` instance.
    ///
    /// The response is written to `dst`. This is called by the server in order
    /// to execute a received command.
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        // Get the value from the shared database state
        // todo
        let response = match db.get(dict::cmd::simple::get::Req { key: self.key })? {
            dict::data_type::DataType::Null => Frame::Null,
            dict::data_type::DataType::String(s) => Frame::OwnedSimple(s),
            dict::data_type::DataType::Bytes(b) => Frame::OwnedBulk(b),
            dict::data_type::DataType::Integer(i) => Frame::OwnedStringSimple(i.to_string()),
            dict::data_type::DataType::Float(i) => Frame::OwnedStringSimple(i.0.to_string()),
            _ => {
                return Err(
                    "WRONGTYPE Operation against a key holding the wrong kind of value".into(),
                )
            }
        };

        Ok(response)
    }
}
