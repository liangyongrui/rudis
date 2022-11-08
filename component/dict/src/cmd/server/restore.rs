use common::now_timestamp_ms;
use keys::Key;
use serde::{Deserialize, Serialize};

use crate::{
    cmd::{ExpiresOp, ExpiresOpResp, ExpiresStatus, ExpiresStatusUpdate, WriteCmd},
    Dict, Value,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Req<'a> {
    pub key: &'a [u8],
    pub value: &'a [u8],
    pub expires_at: u64,
    pub replace: bool,
    pub last_visit_time: u64,
    pub freq: u64,
}

impl<'a> From<Req<'a>> for WriteCmd {
    #[inline]
    fn from(_req: Req<'a>) -> Self {
        // todo
        WriteCmd::None
    }
}

impl<D: Dict> ExpiresOp<(), D> for Req<'_> {
    #[tracing::instrument(skip(dict), level = "debug")]
    fn apply(self, dict: &mut D) -> common::Result<ExpiresOpResp<()>> {
        let mut before = 0;
        if let Some(v) = dict.get(self.key) {
            if !self.replace {
                return Err("BUSYKEY Target key name is busy".into());
            }
            before = v.expires_at;
        }
        let v = Value {
            data: bincode::deserialize(self.value)?,
            expires_at: self.expires_at,
            visit_log: Value::create_visit_log(
                self.last_visit_time / 10,
                Value::get_min(now_timestamp_ms()),
                self.freq,
            ),
        };
        let key: Key = self.key.into();
        dict.insert(key.clone(), v);
        Ok(ExpiresOpResp {
            payload: (),
            expires_status: ExpiresStatus::Update(ExpiresStatusUpdate {
                key,
                before,
                new: self.expires_at,
            }),
        })
    }
}
