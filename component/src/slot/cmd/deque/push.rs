use serde::{Deserialize, Serialize};

use crate::{
    slot::{
        cmd::{Write, WriteCmd},
        data_type::{CollectionType, DataType, Deque, KeyType, SimpleType},
        dict::{self, Dict},
    },
    utils::options::NxXx,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Req {
    pub key: KeyType,
    pub elements: Vec<SimpleType>,
    // true left, false right
    pub left: bool,
    pub nx_xx: NxXx,
}
pub struct Resp {
    /// 原来的大小
    pub old_len: usize,
    /// 更新后大小
    pub new_len: usize,
}

impl From<Req> for WriteCmd {
    fn from(req: Req) -> Self {
        Self::DequePush(req)
    }
}
impl Write<Resp> for Req {
    fn apply(self, id: u64, dict: &mut Dict) -> crate::Result<crate::slot::cmd::WriteResp<Resp>> {
        if let Some(v) = dict.d_get_mut(&self.key) {
            if let DataType::CollectionType(CollectionType::Deque(ref mut deque)) = v.data {
                let old_len = deque.len();
                if self.nx_xx.is_nx() {
                    return Ok(crate::slot::cmd::WriteResp {
                        payload: Resp {
                            new_len: old_len,
                            old_len,
                        },
                        new_expires_at: None,
                    });
                }
                if self.left {
                    for e in self.elements {
                        deque.push_front(e);
                    }
                } else {
                    for e in self.elements {
                        deque.push_back(e);
                    }
                }
                let new_len = deque.len();
                Ok(crate::slot::cmd::WriteResp {
                    payload: Resp { old_len, new_len },
                    new_expires_at: None,
                })
            } else {
                Err("error type".into())
            }
        } else {
            if self.nx_xx.is_xx() {
                return Ok(crate::slot::cmd::WriteResp {
                    payload: Resp {
                        new_len: 0,
                        old_len: 0,
                    },
                    new_expires_at: None,
                });
            }
            let mut deque = Deque::new();
            if self.left {
                for e in self.elements {
                    deque.push_front(e);
                }
            } else {
                for e in self.elements {
                    deque.push_back(e);
                }
            }
            let new_len = deque.len();
            dict.insert(
                self.key,
                dict::Value {
                    id,
                    data: DataType::CollectionType(CollectionType::Deque(deque)),
                    expire_at: None,
                },
            );
            Ok(crate::slot::cmd::WriteResp {
                payload: Resp {
                    new_len,
                    old_len: 0,
                },
                new_expires_at: None,
            })
        }
    }
}

// todo utest
