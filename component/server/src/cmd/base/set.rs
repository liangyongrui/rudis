use common::{
    connection::parse::frame::Frame,
    now_timestamp_ms,
    options::{ExpiresAt, NxXx, SetCmdExpires},
};
use db::Db;
use dict::data_type::DataType;
use keys::Key;
use macros::ParseFrames;

use crate::frame_parse::data_type_to_frame;

/// Set `key` to hold the string `value`.
///
/// <https://redis.io/commands/set>
#[derive(Debug, Clone, ParseFrames)]
pub struct Set {
    /// the lookup key
    pub key: Key,
    /// the value to be stored
    pub value: DataType,
    #[optional]
    pub expires: SetCmdExpires,
    // None not set, true nx, false xx
    #[optional]
    pub nx_xx: NxXx,
    pub get: bool,
}

impl Set {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub async fn apply<'a>(self, db: &Db) -> common::Result<Frame<'a>> {
        let get = self.get;
        let res = db.set(dict::cmd::simple::set::Req {
            key: self.key,
            value: self.value,
            expires_at: match self.expires {
                SetCmdExpires::Ex(e) => ExpiresAt::Specific(now_timestamp_ms() + e * 1000),
                SetCmdExpires::Px(e) => ExpiresAt::Specific(now_timestamp_ms() + e),
                SetCmdExpires::Exat(e) => ExpiresAt::Specific(e * 1000),
                SetCmdExpires::Pxat(e) => ExpiresAt::Specific(e),
                SetCmdExpires::Keepttl => ExpiresAt::Last,
                SetCmdExpires::None => ExpiresAt::Specific(0),
            },
            nx_xx: self.nx_xx,
        })?;
        let response = if get {
            data_type_to_frame(res)
        } else {
            // async drop
            tokio::spawn(async { res });
            Frame::ok()
        };
        Ok(response)
    }
}
