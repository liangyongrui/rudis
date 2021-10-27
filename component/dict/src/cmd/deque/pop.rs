use keys::Key;
use serde::{Deserialize, Serialize};

use crate::{
    cmd::{Write, WriteCmd},
    data_type::DataType,
    Dict,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Req {
    pub key: Key,
    pub count: usize,
    // true left, false right
    pub left: bool,
}

impl From<Req> for WriteCmd {
    fn from(req: Req) -> Self {
        Self::DequePop(req)
    }
}
impl<D: Dict> Write<Vec<DataType>, D> for Req {
    #[tracing::instrument(skip(dict), level = "debug")]
    fn apply(self, dict: &mut D) -> common::Result<Vec<DataType>> {
        if let Some(v) = dict.get(&self.key) {
            return if let DataType::Deque(ref mut deque) = v.data {
                let count = self.count.min(deque.len());
                let res = if self.left {
                    deque.drain(..count).collect()
                } else {
                    let len = deque.len();
                    deque.drain(len - count..).rev().collect()
                };
                Ok(res)
            } else {
                Err("WRONGTYPE Operation against a key holding the wrong kind of value".into())
            };
        }
        Ok(vec![])
    }
}

#[cfg(test)]
mod test {
    use common::options::NxXx;

    use crate::{
        cmd::{
            deque::{pop, push},
            Write,
        },
        MemDict,
    };

    #[test]
    fn test1() {
        let mut dict = MemDict::default();
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
        .apply(&mut dict)
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
        .apply(&mut dict)
        .unwrap();
        assert_eq!(res, vec!["9".into(), "8".into(), "7".into(),]);
        let res = pop::Req {
            key: b"hello"[..].into(),
            count: 4,
            left: true,
        }
        .apply(&mut dict)
        .unwrap();
        assert_eq!(res, vec!["0".into(), "1".into(), "2".into(), "3".into()]);
    }
}
