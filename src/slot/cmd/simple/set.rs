use serde::{Deserialize, Serialize};

use crate::{
    slot::{
        cmd::{Write, WriteCmd, WriteResp},
        data_type::{DataType, SimpleType},
        dict::{self, Dict},
    },
    utils::options::{ExpiresAt, NxXx},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Req {
    pub key: SimpleType,
    pub value: SimpleType,
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
impl Write<SimpleType> for Req {
    fn apply(self, id: u64, dict: &mut Dict) -> crate::Result<WriteResp<SimpleType>> {
        let key = self.key.clone();
        if let Some(v) = dict.d_get(&self.key) {
            if self.nx_xx.is_nx() {
                let old_value = data_type_copy_to_simple(&v.data);
                return Ok(WriteResp {
                    payload: old_value,
                    new_expires_at: None,
                });
            }
            let expire_at = match self.expires_at {
                ExpiresAt::Specific(i) => Some(i),
                ExpiresAt::Last => v.expire_at,
                ExpiresAt::None => None,
            };
            let old = dict
                .insert(
                    self.key,
                    dict::Value {
                        id,
                        data: DataType::SimpleType(self.value),
                        expire_at,
                    },
                )
                .unwrap();
            Ok(WriteResp {
                payload: data_type_to_simple(old.data),
                new_expires_at: expire_at.map(|ea| (ea, key)),
            })
        } else {
            if self.nx_xx.is_xx() {
                return Ok(WriteResp {
                    payload: SimpleType::Null,
                    new_expires_at: None,
                });
            }
            let expire_at = match self.expires_at {
                ExpiresAt::Specific(i) => Some(i),
                _ => None,
            };
            dict.insert(
                self.key,
                dict::Value {
                    id,
                    data: DataType::SimpleType(self.value),
                    expire_at,
                },
            );
            Ok(WriteResp {
                payload: SimpleType::Null,
                new_expires_at: expire_at.map(|ea| (ea, key)),
            })
        }
    }
}
#[inline]
fn data_type_copy_to_simple(dt: &DataType) -> SimpleType {
    match dt {
        DataType::SimpleType(s) => s.clone(),
        DataType::CollectionType(_) => SimpleType::Null,
    }
}

#[inline]
fn data_type_to_simple(dt: DataType) -> SimpleType {
    match dt {
        DataType::SimpleType(s) => s,
        DataType::CollectionType(_) => SimpleType::Null,
    }
}
#[cfg(test)]
mod test {
    use std::thread::sleep;

    use chrono::{Duration, Utc};

    use super::*;
    use crate::slot::{
        cmd::{simple::get, Read},
        dict::Dict,
    };

    #[test]
    fn test1() {
        let mut dict = Dict::new();
        let date_time = Utc::now() + Duration::seconds(1);
        let cmd = Req {
            key: "hello".into(),
            value: "world".into(),
            expires_at: ExpiresAt::Specific(date_time),
            nx_xx: NxXx::None,
        };
        let res = cmd.apply(1, &mut dict).unwrap();
        assert_eq!(
            res,
            WriteResp {
                payload: SimpleType::Null,
                new_expires_at: Some((date_time, "hello".into()))
            }
        );
        let res = get::Req {
            key: &"hello".into(),
        }
        .apply_in_lock(&dict)
        .unwrap();
        assert_eq!(res, "world".into());
        let res = get::Req { key: &"n".into() }.apply_in_lock(&dict).unwrap();
        assert_eq!(res, SimpleType::Null);
        // xx
        let cmd = Req {
            key: "hello".into(),
            value: "world2".into(),
            expires_at: ExpiresAt::Specific(date_time),
            nx_xx: NxXx::Xx,
        };
        let res = cmd.apply(1, &mut dict).unwrap();
        assert_eq!(
            res,
            WriteResp {
                payload: "world".into(),
                new_expires_at: Some((date_time, "hello".into()))
            }
        );
        let cmd = Req {
            key: "n".into(),
            value: "world2".into(),
            expires_at: ExpiresAt::Specific(date_time),
            nx_xx: NxXx::Xx,
        };
        let res = cmd.apply(1, &mut dict).unwrap();
        assert_eq!(
            res,
            WriteResp {
                payload: SimpleType::Null,
                new_expires_at: None,
            }
        );
        let res = get::Req {
            key: &"hello".into(),
        }
        .apply_in_lock(&dict)
        .unwrap();
        assert_eq!(res, "world2".into());
        let res = get::Req { key: &"n".into() }.apply_in_lock(&dict).unwrap();
        assert_eq!(res, SimpleType::Null);
        // nx
        let cmd = Req {
            key: "hello".into(),
            value: "world3".into(),
            expires_at: ExpiresAt::Specific(date_time),
            nx_xx: NxXx::Nx,
        };
        let res = cmd.apply(1, &mut dict).unwrap();
        assert_eq!(
            res,
            WriteResp {
                payload: "world2".into(),
                new_expires_at: None
            }
        );
        let cmd = Req {
            key: "n".into(),
            value: "world3".into(),
            expires_at: ExpiresAt::None,
            nx_xx: NxXx::Nx,
        };
        let res = cmd.apply(1, &mut dict).unwrap();
        assert_eq!(
            res,
            WriteResp {
                payload: SimpleType::Null,
                new_expires_at: None
            }
        );
        let res = get::Req {
            key: &"hello".into(),
        }
        .apply_in_lock(&dict)
        .unwrap();
        assert_eq!(res, "world2".into());
        let res = get::Req { key: &"n".into() }.apply_in_lock(&dict).unwrap();
        assert_eq!(res, "world3".into());
        // time
        let cmd = Req {
            key: "hello".into(),
            value: "world".into(),
            expires_at: ExpiresAt::Last,
            nx_xx: NxXx::None,
        };
        let res = cmd.apply(1, &mut dict).unwrap();
        assert_eq!(
            res,
            WriteResp {
                payload: "world2".into(),
                new_expires_at: Some((date_time, "hello".into()))
            }
        );
        let res = get::Req {
            key: &"hello".into(),
        }
        .apply_in_lock(&dict)
        .unwrap();
        assert_eq!(res, "world".into());
        let res = get::Req { key: &"n".into() }.apply_in_lock(&dict).unwrap();
        assert_eq!(res, "world3".into());
        sleep(std::time::Duration::from_secs(1));
        let res = get::Req {
            key: &"hello".into(),
        }
        .apply_in_lock(&dict)
        .unwrap();
        assert_eq!(res, SimpleType::Null);
        let res = get::Req { key: &"n".into() }.apply_in_lock(&dict).unwrap();
        assert_eq!(res, "world3".into());
    }
}
