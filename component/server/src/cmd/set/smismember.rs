use db::Db;
use macros::ParseFrames;

use crate::Frame;
/// <https://redis.io/commands/smismember>
#[derive(Debug, ParseFrames)]
pub struct Smismember<'a> {
    pub key: &'a [u8],
    pub values: Vec<&'a [u8]>,
}

impl Smismember<'_> {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let res = db.set_exists(dict::cmd::set::exists::Req {
            key: self.key,
            fields: self.values,
        })?;

        Ok(Frame::Array(
            res.into_iter()
                .map(|f| if f { 1 } else { 0 })
                .map(Frame::Integer)
                .collect(),
        ))
    }
}
