pub mod del;
pub mod exists;
pub mod get;
pub mod get_all;
pub mod incr;
pub mod set;

#[cfg(test)]
#[allow(clippy::too_many_lines)]
mod test {

    use common::options::NxXx;

    use super::*;
    use crate::{
        cmd::{Read, Write},
        data_type::DataType,
        Dict,
    };

    #[test]
    fn test1() {
        let mut dict = Dict::default();
        let res = set::Req {
            key: b"hello"[..].into(),
            entries: vec![
                (b"k1"[..].into(), b"v1"[..].into()),
                (b"k2"[..].into(), b"v2"[..].into()),
                (b"k3"[..].into(), b"v3"[..].into()),
            ],
            nx_xx: NxXx::None,
        }
        .apply(&mut dict)
        .unwrap();
        assert_eq!(
            res,
            set::Resp {
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
                let mut v = res
                    .into_iter()
                    .map(|kv| (kv.0.clone(), kv.1))
                    .collect::<Vec<_>>();
                v.sort_unstable_by_key(|t| t.0.clone());
                v
            },
            vec![
                (b"k1"[..].into(), b"v1"[..].into()),
                (b"k2"[..].into(), b"v2"[..].into()),
                (b"k3"[..].into(), b"v3"[..].into()),
            ]
        );
        let res = set::Req {
            key: b"hello"[..].into(),
            entries: vec![
                (b"k1"[..].into(), b"v1"[..].into()),
                (b"k4"[..].into(), b"v4"[..].into()),
                (b"k5"[..].into(), b"v5"[..].into()),
            ],
            nx_xx: NxXx::Nx,
        }
        .apply(&mut dict)
        .unwrap();
        assert_eq!(
            res,
            set::Resp {
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
                let mut v = res
                    .into_iter()
                    .map(|kv| (kv.0.clone(), kv.1))
                    .collect::<Vec<_>>();
                v.sort_unstable_by_key(|t| t.0.clone());
                v
            },
            vec![
                (b"k1"[..].into(), b"v1"[..].into()),
                (b"k2"[..].into(), b"v2"[..].into()),
                (b"k3"[..].into(), b"v3"[..].into()),
                (b"k4"[..].into(), b"v4"[..].into()),
                (b"k5"[..].into(), b"v5"[..].into()),
            ]
        );

        let res = get::Req {
            key: b"hello"[..].into(),
            fields: vec![b"k1"[..].into()],
        }
        .apply(&dict)
        .unwrap();
        assert_eq!(res, vec![b"v1"[..].into()]);
        let res = get::Req {
            key: b"hello"[..].into(),
            fields: vec![b"k6"[..].into()],
        }
        .apply(&dict)
        .unwrap();
        assert_eq!(res, vec![DataType::Null]);
        let res = get::Req {
            key: b"hello2"[..].into(),
            fields: vec![b"k1"[..].into()],
        }
        .apply(&dict)
        .unwrap();
        assert_eq!(res, vec![DataType::Null]);

        let res = exists::Req {
            key: b"hello"[..].into(),
            field: b"k1"[..].into(),
        }
        .apply(&dict)
        .unwrap();
        assert!(res);

        let res = del::Req {
            key: b"hello"[..].into(),
            fields: vec![b"k1"[..].into(), b"k10"[..].into()],
        }
        .apply(&mut dict)
        .unwrap();
        assert_eq!(
            res,
            del::Resp {
                old_len: 5,
                new_len: 4
            }
        );

        let res = exists::Req {
            key: b"hello"[..].into(),
            field: b"k1"[..].into(),
        }
        .apply(&dict)
        .unwrap();
        assert!(!res);
    }
}
