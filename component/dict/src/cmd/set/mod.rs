pub mod add;
pub mod exists;
pub mod get_all;
pub mod remove;

#[cfg(test)]
mod test {
    use std::convert::TryInto;

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
            members: vec!["k1".into(), "k2".into(), "k3".into()],
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
                v.sort_unstable_by_key::<String, _>(|t| t.try_into().unwrap());
                v
            },
            vec!["k1".to_owned(), "k2".to_owned(), "k3".to_owned()]
        );
        let res = add::Req {
            key: b"hello"[..].into(),
            members: vec!["k1".into(), "k4".into(), "k5".into()],
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
            fields: vec!["k1"],
        }
        .apply(&dict)
        .unwrap();
        assert!(res[0]);

        let res = remove::Req {
            key: b"hello"[..].into(),
            members: vec!["k1".into(), "k10".into()],
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
            fields: vec!["k1"],
        }
        .apply(&dict)
        .unwrap();
        assert!(!res[0]);
    }
}
