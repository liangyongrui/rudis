use db::Db;
use macros::ParseFrames;

use crate::{frame_parse, Frame};

/// <https://redis.io/commands/hget>
#[derive(Debug, ParseFrames)]
pub struct Hget<'a> {
    pub key: &'a [u8],
    pub field: &'a [u8],
}

impl Hget<'_> {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        Ok(frame_parse::data_type_to_frame(
            db.kvp_get(dict::cmd::kvp::get::Req {
                key: self.key,
                fields: vec![self.field],
            })?
            .into_iter()
            .next()
            .unwrap(),
        ))
    }
}
