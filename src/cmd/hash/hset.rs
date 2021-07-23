use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{db2::Db, slot::data_type::SimpleType, utils::other_type::SimpleTypePair, Frame};
/// https://redis.io/commands/hset
#[derive(Debug, Clone, ParseFrames)]
pub struct Hset {
    pub key: SimpleType,
    pub pairs: Vec<SimpleTypePair>,
}

impl From<Hset> for crate::slot::cmd::kvp::set::Req {
    fn from(old: Hset) -> Self {
        Self {
            key: old.key,
            entries: old.pairs.into_iter().map(|t| (t.key, t.value)).collect(),
            nx_xx: crate::utils::options::NxXx::None,
        }
    }
}

impl Hset {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let res = db.kvp_set(self.into()).await?;
        Ok(Frame::Integer((res.new_len - res.old_len) as _))
    }
}
