pub mod add;
pub mod exists;
pub mod get_all;
pub mod remove;

#[cfg(test)]
mod test {
    use std::borrow::BorrowMut;

    use parking_lot::RwLock;

    use super::*;
    use crate::slot::{dict::Dict, Read, Write};

    #[test]
    fn test1() {
        let dict = RwLock::new(Dict::new());
        let res = add::Req {
            key: "hello".into(),
            members: vec!["k1".into(), "k2".into(), "k3".into()],
        }
        .apply(1, dict.write().borrow_mut())
        .unwrap()
        .payload;
        assert_eq!(
            res,
            add::Resp {
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
                let mut v = res.into_iter().cloned().collect::<Vec<_>>();
                v.sort();
                v
            },
            {
                let mut v = vec!["k1".into(), "k2".into(), "k3".into()];
                v.sort();
                v
            }
        );
        let res = add::Req {
            key: "hello".into(),
            members: vec!["k1".into(), "k4".into(), "k5".into()],
        }
        .apply(1, dict.write().borrow_mut())
        .unwrap()
        .payload;
        assert_eq!(
            res,
            add::Resp {
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
                let mut v = res.into_iter().cloned().collect::<Vec<_>>();
                v.sort();
                v
            },
            {
                let mut v = vec![
                    "k1".into(),
                    "k2".into(),
                    "k3".into(),
                    "k4".into(),
                    "k5".into(),
                ];
                v.sort();
                v
            }
        );

        let res = exists::Req {
            key: &"hello".into(),
            field: &"k1".into(),
        }
        .apply(&dict)
        .unwrap();
        assert!(res);

        let res = remove::Req {
            key: "hello".into(),
            members: vec!["k1".into(), "k10".into()],
        }
        .apply(1, dict.write().borrow_mut())
        .unwrap()
        .payload;
        assert_eq!(
            res,
            remove::Resp {
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
