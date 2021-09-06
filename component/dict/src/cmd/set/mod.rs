pub mod add;
pub mod exists;
pub mod get_all;
pub mod remove;

#[cfg(test)]
mod test {

    use super::*;
    use crate::{
        cmd::{Read, Write},
        Dict,
    };

    #[test]
    fn test1() {
        let mut dict = Dict::default();
        let res = add::Req {
            key: b"hello"[..].into(),
            members: vec![b"k1"[..].into(), b"k2"[..].into(), b"k3"[..].into()],
        }
        .apply(&mut dict)
        .unwrap();
        assert_eq!(
            res,
            add::Resp {
                old_len: 0,
                new_len: 3
            }
        );
        let res = get_all::Req {
            key: b"hello"[..].into(),
        }
        .apply(&dict)
        .unwrap();
        assert_eq!(
            {
                let mut v = res.into_iter().collect::<Vec<_>>();
                v.sort();
                // v.sort_unstable_by_key::<String, _>(|t| t.try_into().unwrap());
                v
            },
            vec![b"k1"[..].into(), b"k2"[..].into(), b"k3"[..].into()]
        );
        let res = add::Req {
            key: b"hello"[..].into(),
            members: vec![b"k1"[..].into(), b"k4"[..].into(), b"k5"[..].into()],
        }
        .apply(&mut dict)
        .unwrap();
        assert_eq!(
            res,
            add::Resp {
                old_len: 3,
                new_len: 5
            }
        );

        let res = get_all::Req {
            key: b"hello"[..].into(),
        }
        .apply(&dict)
        .unwrap();
        assert_eq!(
            {
                let mut v = res.into_iter().collect::<Vec<_>>();
                v.sort();
                // v.sort_unstable_by_key::<String, _>(|t| t.try_into().unwrap());
                v
            },
            vec![
                b"k1"[..].into(),
                b"k2"[..].into(),
                b"k3"[..].into(),
                b"k4"[..].into(),
                b"k5"[..].into(),
            ]
        );

        let res = exists::Req {
            key: b"hello"[..].into(),
            fields: vec![&b"k1"[..]],
        }
        .apply(&dict)
        .unwrap();
        assert!(res[0]);

        let res = remove::Req {
            key: b"hello"[..].into(),
            members: vec![b"k1"[..].into(), b"k10"[..].into()],
        }
        .apply(&mut dict)
        .unwrap();
        assert_eq!(
            res,
            remove::Resp {
                old_len: 5,
                new_len: 4
            }
        );

        let res = exists::Req {
            key: b"hello"[..].into(),
            fields: vec![&b"k1"[..]],
        }
        .apply(&dict)
        .unwrap();
        assert!(!res[0]);
    }
}
