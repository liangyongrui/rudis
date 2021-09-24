use common::options::{ExpiresAt, NxXx};
use keys::Key;
use serde::{Deserialize, Serialize};

use crate::{
    cmd::{ExpiresOp, ExpiresOpResp, ExpiresStatus, ExpiresStatusUpdate, WriteCmd},
    data_type::DataType,
    Dict, Value,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Req {
    pub key: Key,
    pub value: DataType,
    pub expires_at: ExpiresAt,
    pub nx_xx: NxXx,
}
impl From<Req> for WriteCmd {
    fn from(req: Req) -> Self {
        Self::Set(req)
    }
}

impl<D: Dict> ExpiresOp<DataType, D> for Req {
    #[tracing::instrument(skip(dict), level = "debug")]
    fn apply(self, dict: &mut D) -> common::Result<ExpiresOpResp<DataType>> {
        if let (NxXx::None, ExpiresAt::Specific(expires_at)) = (self.nx_xx, self.expires_at) {
            let (old, expires_status) = if expires_at > 0 {
                let old = dict.insert(
                    self.key.clone(),
                    Value {
                        data: self.value,
                        expires_at,
                        visit_log: Value::new_visit_log(),
                    },
                );
                (
                    old,
                    ExpiresStatus::Update(ExpiresStatusUpdate {
                        key: self.key,
                        before: 0,
                        new: expires_at,
                    }),
                )
            } else {
                let old = dict.insert(
                    self.key,
                    Value {
                        data: self.value,
                        expires_at,
                        visit_log: Value::new_visit_log(),
                    },
                );
                (old, ExpiresStatus::None)
            };
            return Ok(ExpiresOpResp {
                payload: old.map_or(DataType::Null, |v| v.data),
                expires_status,
            });
        }

        let key = self.key.clone();
        if let Some(old) = dict.get(&self.key) {
            if self.nx_xx.is_nx() {
                return Ok(ExpiresOpResp {
                    payload: old.data.clone(),
                    expires_status: ExpiresStatus::None,
                });
            }
            let old_expires_at = old.expires_at;
            let expires_at = match self.expires_at {
                ExpiresAt::Specific(i) => i,
                ExpiresAt::Last => old_expires_at,
            };
            let old = dict.insert(
                self.key.clone(),
                Value {
                    data: self.value,
                    expires_at,
                    visit_log: Value::new_visit_log(),
                },
            );
            let expires_status = ExpiresStatus::Update(ExpiresStatusUpdate {
                key,
                before: old_expires_at,
                new: expires_at,
            });
            Ok(ExpiresOpResp {
                payload: old.map_or(DataType::Null, |v| v.data),
                expires_status,
            })
        } else {
            if self.nx_xx.is_xx() {
                return Ok(ExpiresOpResp {
                    payload: DataType::Null,
                    expires_status: ExpiresStatus::None,
                });
            }
            let expires_at = match self.expires_at {
                ExpiresAt::Specific(i) => i,
                ExpiresAt::Last => 0,
            };

            dict.insert(
                self.key.clone(),
                Value {
                    data: self.value,
                    expires_at,
                    visit_log: Value::new_visit_log(),
                },
            );

            let expires_status = if expires_at == 0 {
                ExpiresStatus::None
            } else {
                ExpiresStatus::Update(ExpiresStatusUpdate {
                    key,
                    before: 0,
                    new: expires_at,
                })
            };
            Ok(ExpiresOpResp {
                payload: DataType::Null,
                expires_status,
            })
        }
    }
}

#[cfg(test)]
#[allow(clippy::too_many_lines)]
mod test {
    use std::thread::sleep;

    use common::now_timestamp_ms;

    use super::*;
    use crate::{
        cmd::{simple::get, Read},
        MemDict,
    };

    #[test]
    fn test1() {
        let mut dict = MemDict::default();
        let date_time = now_timestamp_ms() + 1000;
        let cmd = Req {
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
                    new: date_time
                }),
            }
        );
        let res = get::Req {
            key: b"hello"[..].into(),
        }
        .apply(&mut dict)
        .unwrap();
        assert_eq!(res, "world".into());
        let res = get::Req {
            key: b"n"[..].into(),
        }
        .apply(&mut dict)
        .unwrap();
        assert_eq!(res, DataType::Null);
        // xx
        let cmd = Req {
            key: b"hello"[..].into(),
            value: "world2".into(),
            expires_at: ExpiresAt::Specific(date_time),
            nx_xx: NxXx::Xx,
        };
        let res = cmd.apply(&mut dict).unwrap();
        assert_eq!(
            res,
            ExpiresOpResp {
                payload: "world".into(),
                expires_status: ExpiresStatus::Update(ExpiresStatusUpdate {
                    key: b"hello"[..].into(),
                    before: date_time,
                    new: date_time
                }),
            }
        );
        let cmd = Req {
            key: b"n"[..].into(),
            value: "world2".into(),
            expires_at: ExpiresAt::Specific(date_time),
            nx_xx: NxXx::Xx,
        };
        let res = cmd.apply(&mut dict).unwrap();
        assert_eq!(
            res,
            ExpiresOpResp {
                payload: DataType::Null,
                expires_status: ExpiresStatus::None,
            },
        );
        let res = get::Req {
            key: b"hello"[..].into(),
        }
        .apply(&mut dict)
        .unwrap();
        assert_eq!(res, "world2".into());
        let res = get::Req {
            key: b"n"[..].into(),
        }
        .apply(&mut dict)
        .unwrap();
        assert_eq!(res, DataType::Null);
        // nx
        let cmd = Req {
            key: b"hello"[..].into(),
            value: "world3".into(),
            expires_at: ExpiresAt::Specific(date_time),
            nx_xx: NxXx::Nx,
        };
        let res = cmd.apply(&mut dict).unwrap();
        assert_eq!(
            res,
            ExpiresOpResp {
                payload: "world2".into(),
                expires_status: ExpiresStatus::None,
            },
        );
        let cmd = Req {
            key: b"n"[..].into(),
            value: "world3".into(),
            expires_at: ExpiresAt::Specific(0),
            nx_xx: NxXx::Nx,
        };
        let res = cmd.apply(&mut dict).unwrap();
        assert_eq!(
            res,
            ExpiresOpResp {
                payload: DataType::Null,
                expires_status: ExpiresStatus::None,
            }
        );
        let res = get::Req {
            key: b"hello"[..].into(),
        }
        .apply(&mut dict)
        .unwrap();
        assert_eq!(res, "world2".into());
        let res = get::Req {
            key: b"n"[..].into(),
        }
        .apply(&mut dict)
        .unwrap();
        assert_eq!(res, "world3".into());
        // time
        let cmd = Req {
            key: b"hello"[..].into(),
            value: "world".into(),
            expires_at: ExpiresAt::Last,
            nx_xx: NxXx::None,
        };
        let res = cmd.apply(&mut dict).unwrap();
        assert_eq!(
            res,
            ExpiresOpResp {
                payload: "world2".into(),
                expires_status: ExpiresStatus::Update(ExpiresStatusUpdate {
                    key: b"hello"[..].into(),
                    before: date_time,
                    new: date_time
                }),
            }
        );
        let res = get::Req {
            key: b"hello"[..].into(),
        }
        .apply(&mut dict)
        .unwrap();
        assert_eq!(res, "world".into());
        let res = get::Req {
            key: b"n"[..].into(),
        }
        .apply(&mut dict)
        .unwrap();
        assert_eq!(res, "world3".into());
        sleep(std::time::Duration::from_secs(1));
        let res = get::Req {
            key: b"hello"[..].into(),
        }
        .apply(&mut dict)
        .unwrap();
        assert_eq!(res, DataType::Null);
        let res = get::Req {
            key: b"n"[..].into(),
        }
        .apply(&mut dict)
        .unwrap();
        assert_eq!(res, "world3".into());
    }
}
