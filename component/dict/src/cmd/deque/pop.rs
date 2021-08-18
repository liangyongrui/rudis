use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{
    cmd::{Write, WriteCmd},
    data_type::DataType,
    Dict,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Req {
    pub key: Arc<[u8]>,
    pub count: usize,
    // true left, false right
    pub left: bool,
}

impl From<Req> for WriteCmd {
    fn from(req: Req) -> Self {
        Self::DequePop(req)
    }
}
impl Write<Vec<DataType>> for Req {
    #[tracing::instrument(skip(dict), level = "debug")]
    fn apply(self, dict: &mut Dict) -> common::Result<Vec<DataType>> {
        if let Some(v) = dict.d_get_mut(&self.key) {
            return if let DataType::Deque(ref mut deque) = v.data {
                let mut res = vec![];
                let count = self.count.min(deque.len());
                if self.left {
                    for _ in 0..count {
                        res.push(deque.pop_front().unwrap())
                    }
                } else {
                    for _ in 0..count {
                        res.push(deque.pop_back().unwrap())
                    }
                }
                Ok(res)
            } else {
                Err("error type".into())
            };
        }
        Ok(vec![])
    }
}

#[cfg(test)]
mod test {
    use std::borrow::BorrowMut;

    use common::options::NxXx;
    use parking_lot::RwLock;

    use crate::{
        cmd::{deque::*, Write},
        Dict,
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
                "6".into(),
                "7".into(),
                "8".into(),
                "9".into(),
            ],
            left: false,
            nx_xx: NxXx::None,
        }
        .apply(dict.write().borrow_mut())
        .unwrap();
        assert_eq!(
            res,
            push::Resp {
                old_len: 0,
                new_len: 10
            }
        );

        let res = pop::Req {
            key: b"hello"[..].into(),
            count: 3,
            left: false,
        }
        .apply(dict.write().borrow_mut())
        .unwrap();
        assert_eq!(res, vec!["9".into(), "8".into(), "7".into(),]);
        let res = pop::Req {
            key: b"hello"[..].into(),
            count: 4,
            left: true,
        }
        .apply(dict.write().borrow_mut())
        .unwrap();
        assert_eq!(res, vec!["0".into(), "1".into(), "2".into(), "3".into()]);
    }
}