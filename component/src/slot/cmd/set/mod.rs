pub mod add;
pub mod exists;
pub mod get_all;
pub mod remove;

#[cfg(test)]
mod test {
    use std::{borrow::BorrowMut, convert::TryInto};

    use parking_lot::RwLock;

    use super::*;
    use crate::slot::{dict::Dict, Read, Write};

    #[test]
    fn test1() {
        let dict = RwLock::new(Dict::new());
        let res = add::Req {
            key: b"hello"[..].into(),
            members: vec!["k1".into(), "k2".into(), "k3".into()],
        }
        .apply(dict.write().borrow_mut())
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
        .unwrap()
        .unwrap();
        assert_eq!(
            {
                let mut v = res.into_iter().cloned().collect::<Vec<_>>();
                v.sort_unstable_by_key::<String, _>(|t| t.try_into().unwrap());
                v
            },
            vec!["k1".to_owned(), "k2".to_owned(), "k3".to_owned()]
        );
        let res = add::Req {
            key: b"hello"[..].into(),
            members: vec!["k1".into(), "k4".into(), "k5".into()],
        }
        .apply(dict.write().borrow_mut())
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
        .unwrap()
        .unwrap();
        assert_eq!(
            {
                let mut v = res.into_iter().cloned().collect::<Vec<_>>();
                v.sort_unstable_by_key::<String, _>(|t| t.try_into().unwrap());
                v
            },
            vec![
                "k1".to_owned(),
                "k2".to_owned(),
                "k3".to_owned(),
                "k4".to_owned(),
                "k5".to_owned(),
            ]
        );

        let res = exists::Req {
            key: b"hello"[..].into(),
            field: "k1",
        }
        .apply(&dict)
        .unwrap();
        assert!(res);

        let res = remove::Req {
            key: b"hello"[..].into(),
            members: vec!["k1".into(), "k10".into()],
        }
        .apply(dict.write().borrow_mut())
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
            field: "k1",
        }
        .apply(&dict)
        .unwrap();
        assert!(!res);
    }
}
