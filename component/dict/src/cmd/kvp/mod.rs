pub mod del;
pub mod exists;
pub mod get;
pub mod get_all;
pub mod incr;
pub mod set;

#[cfg(test)]
mod test {
    use std::{borrow::BorrowMut, convert::TryInto};

    use parking_lot::RwLock;

    use super::*;
    use crate::{cmd::Read, cmd::Write, data_type::DataType, Dict};
    use common::options::NxXx;

    #[test]
    fn test1() {
        let dict = RwLock::new(Dict::new());
        let res = set::Req {
            key: b"hello"[..].into(),
            entries: vec![
                ("k1".into(), "v1".into()),
                ("k2".into(), "v2".into()),
                ("k3".into(), "v3".into()),
            ],
            nx_xx: NxXx::None,
        }
        .apply(dict.write().borrow_mut())
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
                v.sort_unstable_by_key::<String, _>(|t| (&t.0).try_into().unwrap());
                v
            },
            vec![
                ("k1".into(), "v1".into()),
                ("k2".into(), "v2".into()),
                ("k3".into(), "v3".into()),
            ]
        );
        let res = set::Req {
            key: b"hello"[..].into(),
            entries: vec![
                ("k1".into(), "v1".into()),
                ("k4".into(), "v4".into()),
                ("k5".into(), "v5".into()),
            ],
            nx_xx: NxXx::Nx,
        }
        .apply(dict.write().borrow_mut())
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
                v.sort_unstable_by_key::<String, _>(|t| (&t.0).try_into().unwrap());
                v
            },
            vec![
                ("k1".into(), "v1".into()),
                ("k2".into(), "v2".into()),
                ("k3".into(), "v3".into()),
                ("k4".into(), "v4".into()),
                ("k5".into(), "v5".into()),
            ]
        );

        let res = get::Req {
            key: b"hello"[..].into(),
            fields: vec!["k1"],
        }
        .apply(&dict)
        .unwrap();
        assert_eq!(res, vec!["v1".into()]);
        let res = get::Req {
            key: b"hello"[..].into(),
            fields: vec!["k6"],
        }
        .apply(&dict)
        .unwrap();
        assert_eq!(res, vec![DataType::Null]);
        let res = get::Req {
            key: b"hello2"[..].into(),
            fields: vec!["k1"],
        }
        .apply(&dict)
        .unwrap();
        assert_eq!(res, vec![DataType::Null]);

        let res = exists::Req {
            key: b"hello"[..].into(),
            field: "k1",
        }
        .apply(&dict)
        .unwrap();
        assert!(res);

        let res = del::Req {
            key: b"hello"[..].into(),
            fields: vec!["k1".into(), "k10".into()],
        }
        .apply(dict.write().borrow_mut())
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
            field: "k1",
        }
        .apply(&dict)
        .unwrap();
        assert!(!res);
    }
}
