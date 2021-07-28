use std::vec;

use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{db::Db, Frame};
/// https://redis.io/commands/hgetall
#[derive(Debug, ParseFrames)]
pub struct Hgetall {
    pub key: Vec<u8>,
}

impl<'a> From<&'a Hgetall> for crate::slot::cmd::kvp::get_all::Req<'a> {
    fn from(old: &'a Hgetall) -> Self {
        Self { key: &old.key }
    }
}

impl Hgetall {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        if let Some(v) = db.kvp_get_all((&self).into())? {
            Ok(Frame::Array(
                v.into_iter()
                    .flat_map(|(k, v)| vec![k.into(), v.into()].into_iter())
                    .collect(),
            ))
        } else {
            Ok(Frame::Array(vec![]))
        }
    }
}
