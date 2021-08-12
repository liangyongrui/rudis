use std::{sync::Arc, vec};

use rcc_macros::ParseFrames;

use crate::{db::Db, Frame};
/// https://redis.io/commands/hgetall
#[derive(Debug, ParseFrames)]
pub struct Hgetall {
    pub key: Arc<[u8]>,
}

impl<'a> From<&'a Hgetall> for crate::slot::cmd::kvp::get_all::Req<'a> {
    fn from(old: &'a Hgetall) -> Self {
        Self { key: &old.key }
    }
}

impl Hgetall {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> crate::Result<Frame> {
        let v = db.kvp_get_all((&self).into())?;
        Ok(Frame::Array(
            v.into_iter()
                .flat_map(|(k, v)| vec![Frame::Simple((&k[..]).into()), v.into()].into_iter())
                .collect(),
        ))
    }
}
