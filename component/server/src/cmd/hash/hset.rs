use std::sync::Arc;

use common::options::NxXx;
use db::Db;
use rcc_macros::ParseFrames;

use crate::{utils::other_type::SimpleTypePair, Frame};

/// https://redis.io/commands/hset
#[derive(Debug, Clone, ParseFrames)]
pub struct Hset {
    pub key: Arc<[u8]>,
    pub pairs: Vec<SimpleTypePair>,
}

impl From<Hset> for dict::cmd::kvp::set::Req {
    fn from(old: Hset) -> Self {
        Self {
            key: old.key,
            entries: old.pairs.into_iter().map(|t| (t.key, t.value)).collect(),
            nx_xx: NxXx::None,
        }
    }
}

impl Hset {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let res = db.kvp_set(self.into())?;
        Ok(Frame::Integer((res.new_len - res.old_len) as _))
    }
}
