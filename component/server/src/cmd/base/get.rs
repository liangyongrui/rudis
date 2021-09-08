use db::Db;
use macros::ParseFrames;

use crate::Frame;

/// Get the value of key.
///
/// If the key does not exist the special value nil is returned. An error is
/// returned if the value stored at key is not a string, because GET only
/// handles string values.
///
/// <https://redis.io/commands/get>
#[derive(Debug, ParseFrames)]
pub struct Get {
    /// Name of the key to get
    pub key: Box<[u8]>,
}

impl<'a> From<&'a Get> for dict::cmd::simple::get::Req<'a> {
    fn from(old: &'a Get) -> Self {
        Self { key: &old.key }
    }
}
impl Get {
    /// Apply the `Get` command to the specified `Db` instance.
    ///
    /// The response is written to `dst`. This is called by the server in order
    /// to execute a received command.
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        // Get the value from the shared database state
        // todo
        let response = match db.get((&self).into())? {
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
