use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{
    slot::{
        cmd::{ExpiresStatus, ExpiresStatusUpdate, ExpiresWrite, ExpiresWriteResp, WriteCmd},
        dict::Dict,
    },
    utils::options::{GtLt, NxXx},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Req {
    pub key: Arc<[u8]>,
    pub expires_at: u64,
    pub nx_xx: NxXx,
    pub gt_lt: GtLt,
}
impl From<Req> for WriteCmd {
    fn from(req: Req) -> Self {
        Self::Expire(req)
    }
}
/// 返回 是否更新成功
impl ExpiresWrite<bool> for Req {
    #[tracing::instrument(skip(dict), level = "debug")]
    fn apply(self, dict: &mut Dict) -> crate::Result<ExpiresWriteResp<bool>> {
        dict.d_get_mut(&self.key).map_or(
            Ok(ExpiresWriteResp {
                payload: false,
                expires_status: ExpiresStatus::None,
            }),
            |v| {
                let update = match (self.nx_xx, self.gt_lt) {
                    (NxXx::None, GtLt::None) => true,
                    (NxXx::Nx, GtLt::None) if v.expires_at == 0 => true,
                    (NxXx::Xx | NxXx::None, GtLt::Gt)
                        if v.expires_at != 0 && v.expires_at < self.expires_at =>
                    {
                        true
                    }
                    (NxXx::Xx | NxXx::None, GtLt::Lt)
                        if v.expires_at == 0 || v.expires_at > self.expires_at =>
                    {
                        true
                    }
                    (NxXx::Xx, GtLt::None) if v.expires_at > 0 => true,
                    _ => false,
                };
                if update {
                    let expires_status = ExpiresStatus::Update(ExpiresStatusUpdate {
                        key: self.key,
                        before: v.expires_at,
                        new: self.expires_at,
                    });
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
            },
        )
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
        utils::options::{ExpiresAt, GtLt, NxXx},
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
        let res = cmd.apply(dict.write().borrow_mut()).unwrap();
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
            nx_xx: NxXx::None,
            gt_lt: GtLt::None,
        };
        let res = cmd.apply(dict.write().borrow_mut()).unwrap();
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
