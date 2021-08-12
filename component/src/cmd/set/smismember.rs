use std::sync::Arc;

use rcc_macros::ParseFrames;

use crate::{Db, Frame};
/// https://redis.io/commands/smismember
#[derive(Debug, ParseFrames)]
pub struct Smismember {
    pub key: Arc<[u8]>,
    pub values: Vec<String>,
}

impl<'a> From<&'a Smismember> for crate::slot::cmd::set::exists::Req<'a> {
    fn from(old: &'a Smismember) -> Self {
        Self {
            key: &old.key,
            fields: old.values.iter().map(std::string::String::as_str).collect(),
        }
    }
}

impl Smismember {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> crate::Result<Frame> {
        let res = db.set_exists((&self).into())?;

        Ok(Frame::Array(
            res.into_iter()
                .map(|f| if f { 1 } else { 0 })
                .map(Frame::Integer)
                .collect(),
        ))
    }
}
