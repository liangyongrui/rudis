pub mod del;
pub mod exists;
pub mod get;
pub mod get_all;
pub mod incr;
pub mod set;

#[cfg(test)]
mod test {
    use std::borrow::BorrowMut;

    use parking_lot::RwLock;

    use super::*;
    use crate::{
        slot::{data_type::SimpleType, dict::Dict, Read, Write},
        utils::options::NxXx,
    };

    #[test]
    fn test1() {
        let dict = RwLock::new(Dict::new());
        let res = set::Req {
            key: "hello".into(),
            entries: vec![
                ("k1".into(), "v1".into()),
                ("k2".into(), "v2".into()),
                ("k3".into(), "v3".into()),
            ],
            nx_xx: NxXx::None,
        }
        .apply(1, dict.write().borrow_mut())
        .unwrap()
        .payload;
        assert_eq!(
            res,
            set::Resp {
                old_len: 0,
                new_len: 3
            }
        );
        let res = get_all::Req {
            key: &"hello".into(),
        }
        .apply(&dict)
        .unwrap()
        .unwrap();
        assert_eq!(
            {
                let mut v = res
                    .into_iter()
                    .map(|kv| (kv.0.clone(), kv.1.clone()))
                    .collect::<Vec<_>>();
                v.sort_unstable();
                v
            },
            {
                let mut v = vec![
                    ("k1".into(), "v1".into()),
                    ("k2".into(), "v2".into()),
                    ("k3".into(), "v3".into()),
                ];
                v.sort_unstable();
                v
            }
        );
        let res = set::Req {
            key: "hello".into(),
            entries: vec![
                ("k1".into(), "v1".into()),
                ("k4".into(), "v4".into()),
                ("k5".into(), "v5".into()),
            ],
            nx_xx: NxXx::Nx,
        }
        .apply(1, dict.write().borrow_mut())
        .unwrap()
        .payload;
        assert_eq!(
            res,
            set::Resp {
                old_len: 3,
                new_len: 5
            }
        );

        let res = get_all::Req {
            key: &"hello".into(),
        }
        .apply(&dict)
        .unwrap()
        .unwrap();
        assert_eq!(
            {
                let mut v = res
                    .into_iter()
                    .map(|kv| (kv.0.clone(), kv.1.clone()))
                    .collect::<Vec<_>>();
                v.sort_unstable();
                v
            },
            {
                let mut v = vec![
                    ("k1".into(), "v1".into()),
                    ("k2".into(), "v2".into()),
                    ("k3".into(), "v3".into()),
                    ("k4".into(), "v4".into()),
                    ("k5".into(), "v5".into()),
                ];
                v.sort_unstable();
                v
            }
        );

        let res = get::Req {
            key: &"hello".into(),
            field: &"k1".into(),
        }
        .apply(&dict)
        .unwrap();
        assert_eq!(res, "v1".into());
        let res = get::Req {
            key: &"hello".into(),
            field: &"k6".into(),
        }
        .apply(&dict)
        .unwrap();
        assert_eq!(res, SimpleType::Null);
        let res = get::Req {
            key: &"hello2".into(),
            field: &"k1".into(),
        }
        .apply(&dict)
        .unwrap();
        assert_eq!(res, SimpleType::Null);

        let res = exists::Req {
            key: &"hello".into(),
            field: &"k1".into(),
        }
        .apply(&dict)
        .unwrap();
        assert!(res);

        let res = del::Req {
            key: "hello".into(),
            fields: vec!["k1".into(), "k10".into()],
        }
        .apply(1, dict.write().borrow_mut())
        .unwrap()
        .payload;
        assert_eq!(
            res,
            del::Resp {
                old_len: 5,
                new_len: 4
            }
        );

        let res = exists::Req {
            key: &"hello".into(),
            field: &"k1".into(),
        }
        .apply(&dict)
        .unwrap();
        assert!(!res);
    }
}
