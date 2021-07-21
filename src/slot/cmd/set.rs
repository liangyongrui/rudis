use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::Write;
use crate::{
    slot::{
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
    #[test]
    fn test1() {
        
    }
}