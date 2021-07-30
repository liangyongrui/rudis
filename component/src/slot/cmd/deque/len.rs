use parking_lot::RwLock;

use crate::slot::{
    cmd::Read,
    data_type::{CollectionType, DataType},
    dict::Dict,
};

#[derive(Debug, Clone)]
pub struct Req<'a> {
    pub key: &'a [u8],
}

impl<'a> Read<usize> for Req<'a> {
    fn apply(self, dict: &RwLock<Dict>) -> crate::Result<usize> {
        if let Some(v) = dict.read().d_get(self.key) {
            if let DataType::CollectionType(CollectionType::Deque(ref deque)) = v.data {
                return Ok(deque.len());
            } else {
                return Err("error type".into());
            }
        }
        Ok(0)
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
        let res = len::Req { key: &b"hello"[..] }.apply(&dict).unwrap();
        assert_eq!(res, 0);
        let res = push::Req {
            key: b"hello"[..].into(),
            elements: vec!["a".into(), "b".into(), "c".into()],
            left: false,
            nx_xx: NxXx::None,
        }
        .apply(1, dict.write().borrow_mut())
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
