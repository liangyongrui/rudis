use db::Db;
use macros::ParseFrames;

use crate::Frame;
/// <https://redis.io/commands/smismember>
#[derive(Debug, ParseFrames)]
pub struct Smismember {
    pub key: Box<[u8]>,
    pub values: Vec<Box<[u8]>>,
}

impl<'a> From<&'a Smismember> for dict::cmd::set::exists::Req<'a> {
    fn from(old: &'a Smismember) -> Self {
        Self {
            key: &old.key,
            fields: old.values.iter().map(|t| &**t).collect(),
        }
    }
}

impl Smismember {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let res = db.set_exists((&self).into())?;

        Ok(Frame::Array(
            res.into_iter()
                .map(|f| if f { 1 } else { 0 })
                .map(Frame::Integer)
                .collect(),
        ))
    }
}
