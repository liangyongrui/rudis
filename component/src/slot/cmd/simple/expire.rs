use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::slot::{
    cmd::{ExpiresStatus, ExpiresWrite, ExpiresWriteResp, WriteCmd},
    dict::Dict,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Req {
    pub key: Arc<[u8]>,
    pub expires_at: Option<DateTime<Utc>>,
}
impl From<Req> for WriteCmd {
    fn from(req: Req) -> Self {
        Self::Expire(req)
    }
}
/// 返回 是否更新成功
impl ExpiresWrite<bool> for Req {
    fn apply(self, id: u64, dict: &mut Dict) -> crate::Result<ExpiresWriteResp<bool>> {
        if let Some(v) = dict.d_get_mut(&self.key) {
            let expires_status = ExpiresStatus::Update {
                key: self.key,
                before: v.expires_at,
                new: self.expires_at,
            };
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

    use chrono::{Duration, Utc};
    use parking_lot::RwLock;

    use crate::{
        slot::{
            cmd::{simple::*, ExpiresStatus, ExpiresWriteResp},
            data_type::SimpleType,
            dict::Dict,
            ExpiresWrite, Read,
        },
        utils::options::{ExpiresAt, NxXx},
    };

    #[test]
    fn test1() {
        let dict = RwLock::new(Dict::new());
        let date_time = Utc::now() + Duration::seconds(1);
        let cmd = set::Req {
            key: b"hello"[..].into(),
            value: "world".into(),
            expires_at: ExpiresAt::None,
            nx_xx: NxXx::None,
        };
        let res = cmd.apply(1, dict.write().borrow_mut()).unwrap();
        assert_eq!(
            res,
            ExpiresWriteResp {
                payload: SimpleType::Null,
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
            expires_at: Some(date_time),
        };
        let res = cmd.apply(1, dict.write().borrow_mut()).unwrap();
        assert_eq!(
            res,
            ExpiresWriteResp {
                payload: true,
                expires_status: ExpiresStatus::Update {
                    key: b"hello"[..].into(),
                    before: None,
                    new: Some(date_time)
                }
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
