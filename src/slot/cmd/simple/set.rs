use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    slot::{
        cmd::Write,
        data_type::{DataType, SimpleType},
        dict::{self, Dict},
    },
    utils::options::{ExpiresAt, NxXx},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Set {
    pub key: SimpleType,
    pub value: SimpleType,
    pub expires_at: ExpiresAt,
    pub nx_xx: NxXx,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Resp {
    /// 原始值
    /// 如果原始值的类型不为SimpleType, 则返回 null
    pub old_value: SimpleType,
    /// 新过期时间
    pub new_expires_at: Option<DateTime<Utc>>,
}

impl Write<Resp> for Set {
    fn apply(self, id: u64, dict: &mut Dict) -> crate::Result<Resp> {
        match dict.inner.entry(self.key) {
            std::collections::hash_map::Entry::Occupied(mut e) => {
                if self.nx_xx.is_nx() {
                    let old_value = data_type_copy_to_simple(&e.get().data);
                    return Ok(Resp {
                        old_value,
                        new_expires_at: None,
                    });
                }
                let expire_at = match self.expires_at {
                    ExpiresAt::Specific(i) => Some(i),
                    ExpiresAt::Last => e.get().expire_at,
                    ExpiresAt::None => None,
                };
                let old = e.insert(dict::Value {
                    id,
                    data: DataType::SimpleType(self.value),
                    expire_at,
                });
                Ok(Resp {
                    old_value: data_type_to_simple(old.data),
                    new_expires_at: expire_at,
                })
            }
            std::collections::hash_map::Entry::Vacant(e) => {
                if self.nx_xx.is_xx() {
                    return Ok(Resp {
                        old_value: SimpleType::Null,
                        new_expires_at: None,
                    });
                }
                let expire_at = match self.expires_at {
                    ExpiresAt::Specific(i) => Some(i),
                    _ => None,
                };
                e.insert(dict::Value {
                    id,
                    data: DataType::SimpleType(self.value),
                    expire_at,
                });
                Ok(Resp {
                    old_value: SimpleType::Null,
                    new_expires_at: expire_at,
                })
            }
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

    use chrono::Duration;

    use super::*;
    use crate::slot::{
        cmd::{
            get::{self, Get},
            Read,
        },
        dict::Dict,
    };

    #[test]
    fn test1() {
        let mut dict = Dict::new();
        let date_time = Utc::now() + Duration::seconds(1);
        let cmd = Set {
            key: "hello".into(),
            value: "world".into(),
            expires_at: ExpiresAt::Specific(date_time),
            nx_xx: NxXx::None,
        };
        let res = cmd.apply(1, &mut dict).unwrap();
        assert_eq!(
            res,
            Resp {
                old_value: SimpleType::Null,
                new_expires_at: Some(date_time)
            }
        );
        let res = Get {
            keys: vec![&"hello".into(), &"n".into()],
        }
        .apply(&dict)
        .unwrap();
        assert_eq!(
            res,
            get::Resp {
                values: vec!["world".into(), SimpleType::Null]
            }
        );
        // xx
        let cmd = Set {
            key: "hello".into(),
            value: "world2".into(),
            expires_at: ExpiresAt::Specific(date_time),
            nx_xx: NxXx::Xx,
        };
        let res = cmd.apply(1, &mut dict).unwrap();
        assert_eq!(
            res,
            Resp {
                old_value: "world".into(),
                new_expires_at: Some(date_time)
            }
        );
        let cmd = Set {
            key: "n".into(),
            value: "world2".into(),
            expires_at: ExpiresAt::Specific(date_time),
            nx_xx: NxXx::Xx,
        };
        let res = cmd.apply(1, &mut dict).unwrap();
        assert_eq!(
            res,
            Resp {
                old_value: SimpleType::Null,
                new_expires_at: None,
            }
        );
        let res = Get {
            keys: vec![&"hello".into(), &"n".into()],
        }
        .apply(&dict)
        .unwrap();
        assert_eq!(
            res,
            get::Resp {
                values: vec!["world2".into(), SimpleType::Null]
            }
        );
        // nx
        let cmd = Set {
            key: "hello".into(),
            value: "world3".into(),
            expires_at: ExpiresAt::Specific(date_time),
            nx_xx: NxXx::Nx,
        };
        let res = cmd.apply(1, &mut dict).unwrap();
        assert_eq!(
            res,
            Resp {
                old_value: "world2".into(),
                new_expires_at: None
            }
        );
        let cmd = Set {
            key: "n".into(),
            value: "world3".into(),
            expires_at: ExpiresAt::None,
            nx_xx: NxXx::Nx,
        };
        let res = cmd.apply(1, &mut dict).unwrap();
        assert_eq!(
            res,
            Resp {
                old_value: SimpleType::Null,
                new_expires_at: None
            }
        );
        let res = Get {
            keys: vec![&"hello".into(), &"n".into()],
        }
        .apply(&dict)
        .unwrap();
        assert_eq!(
            res,
            get::Resp {
                values: vec!["world2".into(), "world3".into(),]
            }
        );
        // time
        let cmd = Set {
            key: "hello".into(),
            value: "world".into(),
            expires_at: ExpiresAt::Last,
            nx_xx: NxXx::None,
        };
        let res = cmd.apply(1, &mut dict).unwrap();
        assert_eq!(
            res,
            Resp {
                old_value: "world2".into(),
                new_expires_at: Some(date_time)
            }
        );
        let res = Get {
            keys: vec![&"hello".into(), &"n".into()],
        }
        .apply(&dict)
        .unwrap();
        assert_eq!(
            res,
            get::Resp {
                values: vec!["world".into(), "world3".into(),]
            }
        );
        sleep(std::time::Duration::from_secs(1));
        let res = Get {
            keys: vec![&"hello".into(), &"n".into()],
        }
        .apply(&dict)
        .unwrap();
        assert_eq!(
            res,
            get::Resp {
                values: vec![SimpleType::Null, "world3".into(),]
            }
        );
    }
}
