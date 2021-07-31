use std::sync::Arc;

use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{db::Db, slot::data_type::DataType, Frame};

/// https://redis.io/commands/hsetnx
#[derive(Debug, ParseFrames, Clone)]
pub struct Hsetnx {
    pub key: Arc<[u8]>,
    pub field: String,
    pub value: DataType,
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
    pub fn apply(self, db: &Db) -> crate::Result<Frame> {
        let res = db.kvp_set(self.into())?;
        Ok(Frame::Integer((res.new_len - res.old_len) as _))
    }
}
