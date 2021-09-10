use db::Db;
use macros::ParseFrames;

use crate::{frame_parse::data_type_to_frame, Frame};

/// <https://redis.io/commands/hmget>
#[derive(Debug, ParseFrames)]
pub struct Hmget<'a> {
    pub key: &'a [u8],
    pub fields: Vec<&'a [u8]>,
}

impl Hmget<'_> {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let v = db.kvp_get(dict::cmd::kvp::get::Req {
            key: self.key,
            fields: self.fields,
        })?;
        let res = v.into_iter().map(data_type_to_frame).collect();
        Ok(Frame::Array(res))
    }
}
