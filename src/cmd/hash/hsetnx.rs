use nom::{bitvec::field, combinator::value};
use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{db2::Db, slot::data_type::SimpleType, Frame};

/// https://redis.io/commands/hsetnx
#[derive(Debug, ParseFrames, Clone)]
pub struct Hsetnx {
    pub key: SimpleType,
    pub field: SimpleType,
    pub value: SimpleType,
}

impl From<Hsetnx> for crate::slot::cmd::kvp::set::Req {
    fn from(old: Hsetnx) -> Self {
        Self {
            key: old.key,
            entries: vec![(old.field, old.value)],
            nx_xx: crate::utils::options::NxXx::Nx,
        }
    }
}

impl Hsetnx {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let res = db.kvp_set(self.into()).await?;
        Ok(Frame::Integer((res.new_len - res.old_len) as _))
    }
}
