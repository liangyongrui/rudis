use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::{
    slot::{
        cmd::{ExpiresStatus, ExpiresStatusUpdate, ExpiresWrite, ExpiresWriteResp, WriteCmd},
        data_type::DataType,
        dict::{self, Dict},
    },
    utils::options::{ExpiresAt, NxXx},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Req {
    pub key: Arc<[u8]>,
    pub value: DataType,
    pub expires_at: ExpiresAt,
    pub nx_xx: NxXx,
}
impl From<Req> for WriteCmd {
    fn from(req: Req) -> Self {
        Self::Set(req)
    }
}
/// 返回 原始值
/// 如果原始值的类型不为SimpleType, 则返回 null
impl ExpiresWrite<DataType> for Req {
    fn apply(self, id: u64, dict: &mut Dict) -> crate::Result<ExpiresWriteResp<DataType>> {
        let key = self.key.clone();
        if let Some(v) = dict.d_get(&self.key) {
            if self.nx_xx.is_nx() {
                return Ok(ExpiresWriteResp {
                    payload: v.data.clone(),
                    expires_status: ExpiresStatus::None,
                });
            }
            let expires_at = match self.expires_at {
                ExpiresAt::Specific(i) => Some(i),
                ExpiresAt::Last => v.expires_at,
                ExpiresAt::None => None,
            };
            let old = dict
                .insert(
                    self.key,
                    dict::Value {
                        id,
                        data: self.value,
                        expires_at,
                    },
                )
                .unwrap();
            let expires_status = ExpiresStatus::Update(ExpiresStatusUpdate {
                key,
                before: old.expires_at,
                new: expires_at,
            });
            Ok(ExpiresWriteResp {
                payload: old.data,
                expires_status,
            })
        } else {
            if self.nx_xx.is_xx() {
                return Ok(ExpiresWriteResp {
                    payload: DataType::Null,
                    expires_status: ExpiresStatus::None,
                });
            }
            let expires_at = match self.expires_at {
                ExpiresAt::Specific(i) => Some(i),
                _ => None,
            };
            // warn!(
            //     "{}, {}, {}, {}, {}, {}, {}, {}, {}, {}",
            //     self.key.len(),
            //     std::mem::size_of::<DataType>(),
            //     std::mem::size_of::<dict::Value>(),
            //     std::mem::size_of::<crate::forward::Message>(),
            //     std::mem::size_of::<bytes::Bytes>(),
            //     std::mem::size_of::<Arc<[u8]>>(),
            //     std::mem::size_of::<&[u8]>(),
            //     std::mem::size_of::<Arc<str>>(),
            //     std::mem::size_of::<&str>(),
            //     std::mem::size_of::<crate::slot::data_type::Kvp>(),
            // );
            dict.insert(
                self.key,
                dict::Value {
                    id,
                    data: self.value,
                    expires_at,
                },
            );

            let expires_status = if expires_at.is_none() {
                ExpiresStatus::None
            } else {
                ExpiresStatus::Update(ExpiresStatusUpdate {
                    key,
                    before: None,
                    new: expires_at,
                })
            };
            Ok(ExpiresWriteResp {
                payload: DataType::Null,
                expires_status,
            })
        }
    }
}

#[cfg(test)]
mod test {
    use std::{borrow::BorrowMut, thread::sleep};

    use chrono::{Duration, Utc};
    use parking_lot::RwLock;

    use super::*;
    use crate::slot::{
        cmd::{simple::get, Read},
        dict::Dict,
    };

    #[test]
    fn test1() {
        let dict = RwLock::new(Dict::new());
        let date_time = Utc::now() + Duration::seconds(1);
        let cmd = Req {
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
                    before: None,
                    new: Some(date_time)
                })
            }
        );
        let res = get::Req {
            key: b"hello"[..].into(),
        }
        .apply(&dict)
        .unwrap();
        assert_eq!(res, "world".into());
        let res = get::Req {
            key: b"n"[..].into(),
        }
        .apply(&dict)
        .unwrap();
        assert_eq!(res, DataType::Null);
        // xx
        let cmd = Req {
            key: b"hello"[..].into(),
            value: "world2".into(),
            expires_at: ExpiresAt::Specific(date_time),
            nx_xx: NxXx::Xx,
        };
        let res = cmd.apply(1, dict.write().borrow_mut()).unwrap();
        assert_eq!(
            res,
            ExpiresWriteResp {
                payload: "world".into(),
                expires_status: ExpiresStatus::Update(ExpiresStatusUpdate {
                    key: b"hello"[..].into(),
                    before: Some(date_time),
                    new: Some(date_time)
                })
            }
        );
        let cmd = Req {
            key: b"n"[..].into(),
            value: "world2".into(),
            expires_at: ExpiresAt::Specific(date_time),
            nx_xx: NxXx::Xx,
        };
        let res = cmd.apply(1, dict.write().borrow_mut()).unwrap();
        assert_eq!(
            res,
            ExpiresWriteResp {
                payload: DataType::Null,
                expires_status: ExpiresStatus::None
            }
        );
        let res = get::Req {
            key: b"hello"[..].into(),
        }
        .apply(&dict)
        .unwrap();
        assert_eq!(res, "world2".into());
        let res = get::Req {
            key: b"n"[..].into(),
        }
        .apply(&dict)
        .unwrap();
        assert_eq!(res, DataType::Null);
        // nx
        let cmd = Req {
            key: b"hello"[..].into(),
            value: "world3".into(),
            expires_at: ExpiresAt::Specific(date_time),
            nx_xx: NxXx::Nx,
        };
        let res = cmd.apply(1, dict.write().borrow_mut()).unwrap();
        assert_eq!(
            res,
            ExpiresWriteResp {
                payload: "world2".into(),
                expires_status: ExpiresStatus::None
            }
        );
        let cmd = Req {
            key: b"n"[..].into(),
            value: "world3".into(),
            expires_at: ExpiresAt::None,
            nx_xx: NxXx::Nx,
        };
        let res = cmd.apply(1, dict.write().borrow_mut()).unwrap();
        assert_eq!(
            res,
            ExpiresWriteResp {
                payload: DataType::Null,
                expires_status: ExpiresStatus::None
            }
        );
        let res = get::Req {
            key: b"hello"[..].into(),
        }
        .apply(&dict)
        .unwrap();
        assert_eq!(res, "world2".into());
        let res = get::Req {
            key: b"n"[..].into(),
        }
        .apply(&dict)
        .unwrap();
        assert_eq!(res, "world3".into());
        // time
        let cmd = Req {
            key: b"hello"[..].into(),
            value: "world".into(),
            expires_at: ExpiresAt::Last,
            nx_xx: NxXx::None,
        };
        let res = cmd.apply(1, dict.write().borrow_mut()).unwrap();
        assert_eq!(
            res,
            ExpiresWriteResp {
                payload: "world2".into(),
                expires_status: ExpiresStatus::Update(ExpiresStatusUpdate {
                    key: b"hello"[..].into(),
                    before: Some(date_time),
                    new: Some(date_time)
                })
            }
        );
        let res = get::Req {
            key: b"hello"[..].into(),
        }
        .apply(&dict)
        .unwrap();
        assert_eq!(res, "world".into());
        let res = get::Req {
            key: b"n"[..].into(),
        }
        .apply(&dict)
        .unwrap();
        assert_eq!(res, "world3".into());
        sleep(std::time::Duration::from_secs(1));
        let res = get::Req {
            key: b"hello"[..].into(),
        }
        .apply(&dict)
        .unwrap();
        assert_eq!(res, DataType::Null);
        let res = get::Req {
            key: b"n"[..].into(),
        }
        .apply(&dict)
        .unwrap();
        assert_eq!(res, "world3".into());
    }
}
