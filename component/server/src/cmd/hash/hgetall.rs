use db::Db;
use macros::ParseFrames;

use crate::{frame_parse::data_type_to_frame, Frame};

/// <https://redis.io/commands/hgetall>
#[derive(Debug, ParseFrames)]
pub struct Hgetall<'a> {
    pub key: &'a [u8],
}

impl Hgetall<'_> {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let v = db.kvp_get_all(dict::cmd::kvp::get_all::Req { key: self.key })?;
        Ok(Frame::Array(
            v.into_iter()
                .flat_map(|(k, v)| {
                    vec![Frame::OwnedBulk(k.into()), data_type_to_frame(v)].into_iter()
                })
                .collect(),
        ))
    }
}
