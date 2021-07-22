use serde::{Deserialize, Serialize};

use crate::{
    slot::{
        cmd::{Write, WriteResp},
        data_type::{CollectionType, DataType, Kvp, SimpleType},
        dict::{self, Dict},
    },
    utils::options::{ExpiresAt, NxXx},
};

/// 追加entries, 如果key 不存在，插入新的再追加
/// nx_xx 根据 kvp 的 key 决定
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Set {
    pub key: SimpleType,
    // key-value list
    pub entries: Vec<(SimpleType, SimpleType)>,
    pub nx_xx: NxXx,
}

pub struct Resp {
    old_len: usize,
    new_len: usize,
}
/// 返回 原始值
/// 如果原始值的类型不为SimpleType, 则返回 null
impl Write<Resp> for Set {
    fn apply(self, id: u64, dict: &mut Dict) -> crate::Result<WriteResp<Resp>> {
        match dict.inner.entry(self.key) {
            std::collections::hash_map::Entry::Occupied(e) => {
                todo!()
            }
            std::collections::hash_map::Entry::Vacant(e) => {
                let new_len = self.entries.len();
                e.insert(dict::Value {
                    expire_at: None,
                    id,
                    data: DataType::CollectionType(CollectionType::Kvp(Kvp::new())),
                });
                Ok(WriteResp {
                    new_expires_at: None,
                    payload: Resp {
                        old_len: 0,
                        new_len,
                    },
                })
            }
        }
        //     match dict.inner.entry(self.key) {
        //         std::collections::hash_map::Entry::Occupied(mut e) => {
        //             if self.nx_xx.is_nx() {
        //                 let old_value = data_type_copy_to_simple(&e.get().data);
        //                 return Ok(WriteResp {
        //                     payload: old_value,
        //                     new_expires_at: None,
        //                 });
        //             }
        //             let expire_at = match self.expires_at {
        //                 ExpiresAt::Specific(i) => Some(i),
        //                 ExpiresAt::Last => e.get().expire_at,
        //                 ExpiresAt::None => None,
        //             };
        //             let old = e.insert(dict::Value {
        //                 id,
        //                 data: DataType::SimpleType(self.value),
        //                 expire_at,
        //             });
        //             Ok(WriteResp {
        //                 payload: data_type_to_simple(old.data),
        //                 new_expires_at: expire_at,
        //             })
        //         }
        //         std::collections::hash_map::Entry::Vacant(e) => {
        //             if self.nx_xx.is_xx() {
        //                 return Ok(WriteResp {
        //                     payload: SimpleType::Null,
        //                     new_expires_at: None,
        //                 });
        //             }
        //             let expire_at = match self.expires_at {
        //                 ExpiresAt::Specific(i) => Some(i),
        //                 _ => None,
        //             };
        //             e.insert(dict::Value {
        //                 id,
        //                 data: DataType::SimpleType(self.value),
        //                 expire_at,
        //             });
        //             Ok(WriteResp {
        //                 payload: SimpleType::Null,
        //                 new_expires_at: expire_at,
        //             })
        //         }
        //     }
        // }
    }
}
