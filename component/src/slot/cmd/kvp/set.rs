use serde::{Deserialize, Serialize};

use crate::{
    slot::{
        cmd::{Write, WriteCmd, WriteResp},
        data_type::{CollectionType, DataType, KeyType, Kvp, SimpleType},
        dict::{self, Dict},
    },
    utils::options::NxXx,
};

/// 追加entries, 如果key 不存在，插入新的再追加
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Req {
    pub key: KeyType,
    // key-value list
    pub entries: Vec<(SimpleType, SimpleType)>,
    /// nx_xx 根据 kvp 的 key 决定
    pub nx_xx: NxXx,
}
#[derive(Debug, PartialEq, Eq)]
pub struct Resp {
    /// 原来的大小
    pub old_len: usize,
    /// 更新后大小
    pub new_len: usize,
}
impl From<Req> for WriteCmd {
    fn from(req: Req) -> Self {
        Self::KvpSet(req)
    }
}
impl Write<Resp> for Req {
    fn apply(self, id: u64, dict: &mut Dict) -> crate::Result<WriteResp<Resp>> {
        let old = dict.d_get_mut_or_insert_with(self.key, || dict::Value {
            id,
            data: DataType::CollectionType(CollectionType::Kvp(Kvp::new())),
            expire_at: None,
        });
        if let DataType::CollectionType(CollectionType::Kvp(ref mut kvp)) = old.data {
            let old_len = kvp.size();
            match self.nx_xx {
                NxXx::Nx => {
                    for (k, v) in self.entries {
                        if !kvp.contains_key(&k) {
                            kvp.insert_mut(k, v)
                        }
                    }
                }
                NxXx::Xx => {
                    for (k, v) in self.entries {
                        if kvp.contains_key(&k) {
                            kvp.insert_mut(k, v)
                        }
                    }
                }
                NxXx::None => {
                    for (k, v) in self.entries {
                        kvp.insert_mut(k, v)
                    }
                }
            }
            Ok(WriteResp {
                new_expires_at: None,
                payload: Resp {
                    old_len,
                    new_len: kvp.size(),
                },
            })
        } else {
            Err("error type".into())
        }
    }
}

#[cfg(test)]
mod test {
    use std::borrow::BorrowMut;

    use parking_lot::RwLock;

    use crate::{
        slot::{cmd::kvp::*, dict::Dict, Read, Write},
        utils::options::NxXx,
    };

    #[test]
    fn test1() {
        let dict = RwLock::new(Dict::new());
        let res = set::Req {
            key: "hello".into(),
            entries: vec![
                ("k1".into(), "v1".into()),
                ("k2".into(), "v2".into()),
                ("k3".into(), "v3".into()),
            ],
            nx_xx: NxXx::None,
        }
        .apply(1, dict.write().borrow_mut())
        .unwrap()
        .payload;
        assert_eq!(
            res,
            set::Resp {
                old_len: 0,
                new_len: 3
            }
        );
        let res = get_all::Req {
            key: &"hello".into(),
        }
        .apply(&dict)
        .unwrap()
        .unwrap();
        assert_eq!(
            {
                let mut v = res
                    .into_iter()
                    .map(|kv| (kv.0.clone(), kv.1.clone()))
                    .collect::<Vec<_>>();
                v.sort();
                v
            },
            {
                let mut v = vec![
                    ("k1".into(), "v1".into()),
                    ("k2".into(), "v2".into()),
                    ("k3".into(), "v3".into()),
                ];
                v.sort();
                v
            }
        );
        let res = set::Req {
            key: "hello".into(),
            entries: vec![
                ("k1".into(), "v1".into()),
                ("k4".into(), "v4".into()),
                ("k5".into(), "v5".into()),
            ],
            nx_xx: NxXx::Nx,
        }
        .apply(1, dict.write().borrow_mut())
        .unwrap()
        .payload;
        assert_eq!(
            res,
            set::Resp {
                old_len: 3,
                new_len: 5
            }
        );

        let res = get_all::Req {
            key: &"hello".into(),
        }
        .apply(&dict)
        .unwrap()
        .unwrap();
        assert_eq!(
            {
                let mut v = res
                    .into_iter()
                    .map(|kv| (kv.0.clone(), kv.1.clone()))
                    .collect::<Vec<_>>();
                v.sort();
                v
            },
            {
                let mut v = vec![
                    ("k1".into(), "v1".into()),
                    ("k2".into(), "v2".into()),
                    ("k3".into(), "v3".into()),
                    ("k4".into(), "v4".into()),
                    ("k5".into(), "v5".into()),
                ];
                v.sort();
                v
            }
        );
    }
}
