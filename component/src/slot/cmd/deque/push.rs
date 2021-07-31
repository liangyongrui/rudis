use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{
    slot::{
        cmd::{Write, WriteCmd},
        data_type::{DataType, Deque},
        dict::{self, Dict},
    },
    utils::options::NxXx,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Req {
    pub key: Arc<[u8]>,
    pub elements: Vec<DataType>,
    // true left, false right
    pub left: bool,
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
        Self::DequePush(req)
    }
}
impl Write<Resp> for Req {
    fn apply(self, id: u64, dict: &mut Dict) -> crate::Result<Resp> {
        if let Some(v) = dict.d_get_mut(&self.key) {
            if let DataType::Deque(ref mut deque) = v.data {
                let old_len = deque.len();
                if self.nx_xx.is_nx() {
                    return Ok(Resp {
                        new_len: old_len,
                        old_len,
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
                Ok(Resp { old_len, new_len })
            } else {
                Err("error type".into())
            }
        } else {
            if self.nx_xx.is_xx() {
                return Ok(Resp {
                    new_len: 0,
                    old_len: 0,
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
                    data: DataType::Deque(deque),
                    expires_at: None,
                },
            );
            Ok(Resp {
                new_len,
                old_len: 0,
            })
        }
    }
}

#[cfg(test)]
mod test {
    use std::borrow::BorrowMut;

    use parking_lot::RwLock;

    use crate::{
        slot::{cmd::deque::*, dict::Dict, Read, Write},
        utils::options::NxXx,
    };

    #[test]
    fn test1() {
        let dict = RwLock::new(Dict::new());
        let res = push::Req {
            key: b"hello"[..].into(),
            elements: vec![
                "0".into(),
                "1".into(),
                "2".into(),
                "3".into(),
                "4".into(),
                "5".into(),
            ],
            left: false,
            nx_xx: NxXx::Xx,
        }
        .apply(1, dict.write().borrow_mut())
        .unwrap();
        assert_eq!(
            res,
            push::Resp {
                old_len: 0,
                new_len: 0
            }
        );
        let res = push::Req {
            key: b"hello"[..].into(),
            elements: vec![
                "0".into(),
                "1".into(),
                "2".into(),
                "3".into(),
                "4".into(),
                "5".into(),
            ],
            left: false,
            nx_xx: NxXx::None,
        }
        .apply(1, dict.write().borrow_mut())
        .unwrap();
        assert_eq!(
            res,
            push::Resp {
                old_len: 0,
                new_len: 6
            }
        );

        let res = range::Req {
            key: b"hello"[..].into(),
            start: 0,
            stop: -1,
        }
        .apply(&dict)
        .unwrap();
        assert_eq!(
            res,
            vec![
                "0".into(),
                "1".into(),
                "2".into(),
                "3".into(),
                "4".into(),
                "5".into(),
            ]
        );

        let res = push::Req {
            key: b"hello"[..].into(),
            elements: vec!["0".into(), "1".into(), "2".into()],
            left: true,
            nx_xx: NxXx::Xx,
        }
        .apply(1, dict.write().borrow_mut())
        .unwrap();
        assert_eq!(
            res,
            push::Resp {
                old_len: 6,
                new_len: 9
            }
        );
        let res = range::Req {
            key: b"hello"[..].into(),
            start: 0,
            stop: -1,
        }
        .apply(&dict)
        .unwrap();
        assert_eq!(
            res,
            vec![
                "2".into(),
                "1".into(),
                "0".into(),
                "0".into(),
                "1".into(),
                "2".into(),
                "3".into(),
                "4".into(),
                "5".into(),
            ]
        );
    }
}
