use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::slot::{
    cmd::{ExpiresStatus, ExpiresStatusUpdate, ExpiresWrite, ExpiresWriteResp, WriteCmd},
    dict::Dict,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Req {
    pub key: Arc<[u8]>,
    pub expires_at: u64,
}
impl From<Req> for WriteCmd {
    fn from(req: Req) -> Self {
        Self::Expire(req)
    }
}
/// 返回 是否更新成功
impl ExpiresWrite<bool> for Req {
    #[tracing::instrument(skip(dict), level = "debug")]
    fn apply(self, id: u64, dict: &mut Dict) -> crate::Result<ExpiresWriteResp<bool>> {
        if let Some(v) = dict.d_get_mut(&self.key) {
            let expires_status = ExpiresStatus::Update(ExpiresStatusUpdate {
                key: self.key,
                before: v.expires_at,
                new: self.expires_at,
            });
            v.id = id;
            v.expires_at = self.expires_at;
            Ok(ExpiresWriteResp {
                payload: true,
                expires_status,
            })
        } else {
            Ok(ExpiresWriteResp {
                payload: false,
                expires_status: ExpiresStatus::None,
            })
        }
    }
}

#[cfg(test)]
mod test {
    use std::{borrow::BorrowMut, thread::sleep};

    use chrono::Duration;
    use parking_lot::RwLock;

    use crate::{
        slot::{
            cmd::{simple::*, ExpiresStatus, ExpiresStatusUpdate, ExpiresWriteResp},
            data_type::DataType,
            dict::Dict,
            ExpiresWrite, Read,
        },
        utils::options::{ExpiresAt, NxXx},
    };

    #[test]
    fn test1() {
        let dict = RwLock::new(Dict::new());
        let date_time = crate::utils::now_timestamp_ms() + 1000;
        let cmd = set::Req {
            key: b"hello"[..].into(),
            value: "world".into(),
            expires_at: ExpiresAt::Specific(0),
            nx_xx: NxXx::None,
        };
        let res = cmd.apply(1, dict.write().borrow_mut()).unwrap();
        assert_eq!(
            res,
            ExpiresWriteResp {
                payload: DataType::Null,
                expires_status: ExpiresStatus::None
            }
        );
        let res = exists::Req {
            key: b"hello"[..].into(),
        }
        .apply(&dict)
        .unwrap();
        assert!(res);

        let cmd = expire::Req {
            key: b"hello"[..].into(),
            expires_at: date_time,
        };
        let res = cmd.apply(1, dict.write().borrow_mut()).unwrap();
        assert_eq!(
            res,
            ExpiresWriteResp {
                payload: true,
                expires_status: ExpiresStatus::Update(ExpiresStatusUpdate {
                    key: b"hello"[..].into(),
                    before: 0,
                    new: date_time
                })
            }
        );
        let res = exists::Req {
            key: b"hello"[..].into(),
        }
        .apply(&dict)
        .unwrap();
        assert!(res);

        sleep(Duration::seconds(1).to_std().unwrap());
        let res = exists::Req {
            key: b"hello"[..].into(),
        }
        .apply(&dict)
        .unwrap();
        assert!(!res);
    }
}
