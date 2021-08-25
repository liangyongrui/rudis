use parking_lot::RwLock;

use crate::{cmd::Read, data_type::DataType, Dict};

#[derive(Debug, Clone)]
pub struct Req<'a> {
    pub key: &'a [u8],
}

impl<'a> Read<usize> for Req<'a> {
    #[tracing::instrument(skip(dict), level = "debug")]
    fn apply(self, dict: &RwLock<Dict>) -> common::Result<usize> {
        if let Some(v) = dict.read().d_get(self.key) {
            return if let DataType::Deque(ref deque) = v.data {
                Ok(deque.len())
            } else {
                Err("WRONGTYPE Operation against a key holding the wrong kind of value".into())
            };
        }
        Ok(0)
    }
}

#[cfg(test)]
mod test {
    use std::borrow::BorrowMut;

    use common::options::NxXx;
    use parking_lot::RwLock;

    use crate::{
        cmd::{deque::*, Read, Write},
        Dict,
    };

    #[test]
    fn test1() {
        let dict = RwLock::new(Dict::new());
        let res = len::Req { key: &b"hello"[..] }.apply(&dict).unwrap();
        assert_eq!(res, 0);
        let res = push::Req {
            key: b"hello"[..].into(),
            elements: vec!["a".into(), "b".into(), "c".into()],
            left: false,
            nx_xx: NxXx::None,
        }
        .apply(dict.write().borrow_mut())
        .unwrap();
        assert_eq!(
            res,
            push::Resp {
                old_len: 0,
                new_len: 3
            }
        );
        let res = len::Req { key: &b"hello"[..] }.apply(&dict).unwrap();
        assert_eq!(res, 3);
    }
}
