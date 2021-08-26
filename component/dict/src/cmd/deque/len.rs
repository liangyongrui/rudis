use crate::{cmd::Read, data_type::DataType, Dict};

#[derive(Debug, Clone)]
pub struct Req<'a> {
    pub key: &'a [u8],
}

impl<'a> Read<usize> for Req<'a> {
    #[tracing::instrument(skip(dict), level = "debug")]
    fn apply(self, dict: &Dict) -> common::Result<usize> {
        if let Some(v) = dict.d_get(self.key) {
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

    use common::options::NxXx;

    use crate::{
        cmd::{deque::*, Read, Write},
        Dict,
    };

    #[test]
    fn test1() {
        let mut dict = Dict::default();
        let res = len::Req { key: &b"hello"[..] }.apply(&dict).unwrap();
        assert_eq!(res, 0);
        let res = push::Req {
            key: b"hello"[..].into(),
            elements: vec!["a".into(), "b".into(), "c".into()],
            left: false,
            nx_xx: NxXx::None,
        }
        .apply(&mut dict)
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
