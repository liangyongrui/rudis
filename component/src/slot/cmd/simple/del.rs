use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::slot::{
    cmd::{ExpiresStatus, ExpiresStatusUpdate, ExpiresWrite, ExpiresWriteResp, WriteCmd},
    dict::{Dict, Value},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Req {
    pub key: Arc<[u8]>,
}

impl From<Req> for WriteCmd {
    fn from(req: Req) -> Self {
        Self::Del(req)
    }
}
/// 返回 原始值
impl ExpiresWrite<Option<Value>> for Req {
    fn apply(self, _id: u64, dict: &mut Dict) -> crate::Result<ExpiresWriteResp<Option<Value>>> {
        if dict.d_exists(&self.key) {
            let res = dict.remove(&self.key);

            let expires_status = res
                .as_ref()
                .map(|v| {
                    if v.expires_at > 0 {
                        ExpiresStatus::Update(ExpiresStatusUpdate {
                            key: self.key,
                            before: v.expires_at,
                            new: 0,
                        })
                    } else {
                        ExpiresStatus::None
                    }
                })
                .unwrap_or(ExpiresStatus::None);

            Ok(ExpiresWriteResp {
                payload: res,
                expires_status,
            })
        } else {
            Ok(ExpiresWriteResp {
                payload: None,
                expires_status: ExpiresStatus::None,
            })
        }
    }
}

#[cfg(test)]
mod test {
    use std::borrow::BorrowMut;

    use parking_lot::RwLock;

    use super::*;
    use crate::{
        slot::{
            cmd::{simple::*, ExpiresStatusUpdate},
            data_type::DataType,
            dict::Dict,
        },
        utils::{
            now_timestamp_ms,
            options::{ExpiresAt, NxXx},
        },
    };

    #[test]
    fn test1() {
        let dict = RwLock::new(Dict::new());
        let res = del::Req {
            key: b"hello"[..].into(),
        }
        .apply(2, dict.write().borrow_mut())
        .unwrap()
        .payload;
        assert_eq!(res, None);
        let date_time = now_timestamp_ms() + 1000;
        let cmd = set::Req {
            key: b"hello"[..].into(),
            value: "world".into(),
            expires_at: ExpiresAt::Specific(date_time),
            nx_xx: NxXx::None,
        };
        let res = cmd.apply(1, dict.write().borrow_mut()).unwrap();
        assert_eq!(
            res,
            ExpiresWriteResp {
                payload: DataType::Null,
                expires_status: ExpiresStatus::Update(ExpiresStatusUpdate {
                    key: b"hello"[..].into(),
                    before: 0,
                    new: date_time,
                })
            }
        );
        let res = del::Req {
            key: b"hello"[..].into(),
        }
        .apply(2, dict.write().borrow_mut())
        .unwrap()
        .payload
        .unwrap()
        .data;
        assert_eq!(res, DataType::String("world".into()));
    }
}
