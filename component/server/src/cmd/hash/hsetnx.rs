use std::sync::Arc;

use common::options::NxXx;
use db::Db;
use dict::data_type::DataType;
use rcc_macros::ParseFrames;

use crate::Frame;

/// https://redis.io/commands/hsetnx
#[derive(Debug, ParseFrames, Clone)]
pub struct Hsetnx {
    pub key: Arc<[u8]>,
    pub field: String,
    pub value: DataType,
}

impl From<Hsetnx> for dict::cmd::kvp::set::Req {
    fn from(old: Hsetnx) -> Self {
        Self {
            key: old.key,
            entries: vec![(old.field, old.value)],
            nx_xx: NxXx::Nx,
        }
    }
}

impl Hsetnx {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let res = db.kvp_set(self.into())?;
        Ok(Frame::Integer((res.new_len - res.old_len) as _))
    }
}
