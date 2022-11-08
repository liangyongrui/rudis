use common::options::{GtLt, NxXx};
use keys::Key;
use serde::{Deserialize, Serialize};

use crate::{
    cmd::{ExpiresOp, ExpiresOpResp, ExpiresStatus, ExpiresStatusUpdate, WriteCmd},
    Dict,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Req {
    pub key: Key,
    pub expires_at: u64,
    pub nx_xx: NxXx,
    pub gt_lt: GtLt,
}
impl From<Req> for WriteCmd {
    #[inline]
    fn from(req: Req) -> Self {
        Self::Expire(req)
    }
}
/// 返回 是否更新成功
impl<D: Dict> ExpiresOp<bool, D> for Req {
    #[tracing::instrument(skip(dict), level = "debug")]
    fn apply(self, dict: &mut D) -> common::Result<ExpiresOpResp<bool>> {
        dict.get(&self.key).map_or(
            Ok(ExpiresOpResp {
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
                    Ok(ExpiresOpResp {
                        payload: true,
                        expires_status,
                    })
                } else {
                    Ok(ExpiresOpResp {
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
    use std::{thread::sleep, time::Duration};

    use common::{
        now_timestamp_ms,
        options::{ExpiresAt, GtLt, NxXx},
    };

    use crate::{
        cmd::{
            simple::{exists, expire, set},
            ExpiresOp, ExpiresOpResp, ExpiresStatus, ExpiresStatusUpdate, Read,
        },
        data_type::DataType,
        MemDict,
    };

    #[test]
    fn test1() {
        let mut dict = MemDict::default();
        let date_time = now_timestamp_ms() + 1000;
        let cmd = set::Req {
            key: b"hello"[..].into(),
            value: "world".into(),
            expires_at: ExpiresAt::Specific(0),
            nx_xx: NxXx::None,
        };
        let res = cmd.apply(&mut dict).unwrap();
        assert_eq!(
            res,
            ExpiresOpResp {
                payload: DataType::Null,
                expires_status: ExpiresStatus::None,
            }
        );
        let res = exists::Req {
            key: b"hello"[..].into(),
        }
        .apply(&mut dict)
        .unwrap();
        assert!(res);

        let cmd = expire::Req {
            key: b"hello"[..].into(),
            expires_at: date_time,
            nx_xx: NxXx::None,
            gt_lt: GtLt::None,
        };
        let res = cmd.apply(&mut dict).unwrap();
        assert_eq!(
            res,
            ExpiresOpResp {
                payload: true,
                expires_status: ExpiresStatus::Update(ExpiresStatusUpdate {
                    key: b"hello"[..].into(),
                    before: 0,
                    new: date_time
                }),
            }
        );
        let res = exists::Req {
            key: b"hello"[..].into(),
        }
        .apply(&mut dict)
        .unwrap();
        assert!(res);

        sleep(Duration::from_secs(1));
        let res = exists::Req {
            key: b"hello"[..].into(),
        }
        .apply(&mut dict)
        .unwrap();
        assert!(!res);
    }
}
