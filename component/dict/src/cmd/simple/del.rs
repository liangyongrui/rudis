use keys::Key;
use serde::{Deserialize, Serialize};

use crate::{
    cmd::{ExpiresOp, ExpiresOpResp, ExpiresStatus, ExpiresStatusUpdate, WriteCmd},
    Dict, Value,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Req {
    pub key: Key,
}

impl From<Req> for WriteCmd {
    fn from(req: Req) -> Self {
        Self::Del(req)
    }
}
/// 返回 原始值
impl<D: Dict> ExpiresOp<Option<Value>, D> for Req {
    #[tracing::instrument(skip(dict), level = "debug")]
    fn apply(self, dict: &mut D) -> common::Result<ExpiresOpResp<Option<Value>>> {
        let res = dict.remove(&self.key).map_or(
            ExpiresOpResp {
                payload: None,
                expires_status: ExpiresStatus::None,
            },
            |v| {
                let expires_status = if v.expires_at > 0 {
                    ExpiresStatus::Update(ExpiresStatusUpdate {
                        key: self.key,
                        before: v.expires_at,
                        new: 0,
                    })
                } else {
                    ExpiresStatus::None
                };

                ExpiresOpResp {
                    payload: Some(v),
                    expires_status,
                }
            },
        );
        Ok(res)
    }
}

#[cfg(test)]
mod test {
    use common::{
        now_timestamp_ms,
        options::{ExpiresAt, NxXx},
    };

    use super::*;
    use crate::{
        cmd::{
            simple::{del, set},
            ExpiresStatusUpdate,
        },
        data_type::DataType,
        MemDict,
    };

    #[test]
    fn test1() {
        let mut dict = MemDict::default();
        let res = del::Req {
            key: b"hello"[..].into(),
        }
        .apply(&mut dict)
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
        let res = cmd.apply(&mut dict).unwrap();
        assert_eq!(
            res,
            ExpiresOpResp {
                payload: DataType::Null,
                expires_status: ExpiresStatus::Update(ExpiresStatusUpdate {
                    key: b"hello"[..].into(),
                    before: 0,
                    new: date_time,
                }),
            }
        );
        let res = del::Req {
            key: b"hello"[..].into(),
        }
        .apply(&mut dict)
        .unwrap()
        .payload
        .unwrap()
        .data;
        assert_eq!(res, DataType::String(b"world"[..].into()));
    }
}
