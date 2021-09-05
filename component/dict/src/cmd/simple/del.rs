use keys::Key;
use serde::{Deserialize, Serialize};

use crate::{
    cmd::{ExpiresStatus, ExpiresStatusUpdate, ExpiresWrite, ExpiresWriteResp, WriteCmd},
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
impl ExpiresWrite<Option<Value>> for Req {
    #[tracing::instrument(skip(dict), level = "debug")]
    fn apply(self, dict: &mut Dict) -> common::Result<ExpiresWriteResp<Option<Value>>> {
        if dict.exists(&self.key) {
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
        Dict,
    };

    #[test]
    fn test1() {
        let mut dict = Dict::default();
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
        .apply(&mut dict)
        .unwrap()
        .payload
        .unwrap()
        .data;
        assert_eq!(res, DataType::String("world".into()));
    }
}
