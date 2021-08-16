use std::{convert::TryInto, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::{
    cmd::{Write, WriteCmd},
    data_type::{DataType, Kvp},
    Dict, Value,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Req {
    pub key: Arc<[u8]>,
    pub field: String,
    pub value: i64,
}
impl From<Req> for WriteCmd {
    fn from(req: Req) -> Self {
        Self::KvpIncr(req)
    }
}

/// 返回 更新后的值
impl Write<i64> for Req {
    #[tracing::instrument(skip(dict), level = "debug")]
    fn apply(self, dict: &mut Dict) -> common::Result<i64> {
        let v = dict.d_get_mut_or_insert_with(&self.key, || Value {
            expires_at: 0,
            data: DataType::Kvp(Box::new(Kvp::new())),
        });
        match v.data {
            DataType::Kvp(ref mut kvp) => {
                if let Some(s) = kvp.get_mut(&self.field) {
                    let old: i64 = (&*s).try_into()?;
                    let new = old + self.value;
                    *s = DataType::Integer(new);
                    Ok(new)
                } else {
                    kvp.insert(self.field, DataType::Integer(self.value));
                    Ok(self.value)
                }
            }
            _ => Err("error type".into()),
        }
    }
}

#[cfg(test)]
mod test {
    use std::borrow::BorrowMut;

    use common::options::NxXx;
    use parking_lot::RwLock;

    use crate::{
        cmd::{kvp::*, Read, Write},
        Dict,
    };

    #[test]
    fn test1() {
        let dict = RwLock::new(Dict::new());
        let res = set::Req {
            key: b"hello"[..].into(),
            entries: vec![("k1".into(), "1".into()), ("k2".into(), "2".into())],
            nx_xx: NxXx::None,
        }
        .apply(dict.write().borrow_mut())
        .unwrap();
        assert_eq!(
            res,
            set::Resp {
                old_len: 0,
                new_len: 2
            }
        );

        let res = incr::Req {
            key: b"hello"[..].into(),
            field: "k1".into(),
            value: 1,
        }
        .apply(dict.write().borrow_mut())
        .unwrap();
        assert_eq!(res, 2);

        let res = incr::Req {
            key: b"hello"[..].into(),
            field: "k3".into(),
            value: 10,
        }
        .apply(dict.write().borrow_mut())
        .unwrap();
        assert_eq!(res, 10);

        let res = get_all::Req {
            key: b"hello"[..].into(),
        }
        .apply(&dict)
        .unwrap();
        assert_eq!(
            {
                let mut v = res
                    .into_iter()
                    .map(|kv| (kv.0, kv.1))
                    .collect::<Vec<(String, _)>>();
                v.sort_unstable_by_key(|t| t.0.clone());
                v
            },
            vec![
                ("k1".to_owned(), 2.into()),
                ("k2".to_owned(), "2".into()),
                ("k3".to_owned(), 10.into())
            ]
        );
    }
}
