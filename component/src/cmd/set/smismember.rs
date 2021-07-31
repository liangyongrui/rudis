use std::sync::Arc;

use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{Db, Frame};
/// https://redis.io/commands/smismember
#[derive(Debug, ParseFrames)]
pub struct Smismember {
    pub key: Arc<[u8]>,
    pub values: Vec<String>,
}

impl<'a> From<&'a Smismember> for crate::slot::cmd::set::get_all::Req<'a> {
    fn from(old: &'a Smismember) -> Self {
        Self { key: &old.key }
    }
}

impl Smismember {
    #[instrument(skip(self, db))]
    pub fn apply(self, db: &Db) -> crate::Result<Frame> {
        if let Some(res) = db.set_get_all((&self).into())? {
            Ok(Frame::Array(
                self.values
                    .iter()
                    .map(|f| if res.contains(f) { 1 } else { 0 })
                    .map(Frame::Integer)
                    .collect(),
            ))
        } else {
            Ok(Frame::Array(vec![Frame::Null; self.values.len()]))
        }
    }
}
