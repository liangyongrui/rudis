use std::{convert::TryInto, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::slot::{
    cmd::{Write, WriteCmd},
    data_type::{CollectionType, DataType, Kvp, SimpleType},
    dict::{self, Dict},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Req {
    pub key: Arc<[u8]>,
    pub field: SimpleType,
    pub value: i64,
}
impl From<Req> for WriteCmd {
    fn from(req: Req) -> Self {
        Self::KvpIncr(req)
    }
}

/// 返回 更新后的值
impl Write<i64> for Req {
    fn apply(self, id: u64, dict: &mut Dict) -> crate::Result<i64> {
        let v = dict.d_get_mut_or_insert_with(self.key, || dict::Value {
            expires_at: None,
            id,
            data: DataType::CollectionType(CollectionType::Kvp(Kvp::new())),
        });
        match v.data {
            crate::slot::data_type::DataType::CollectionType(CollectionType::Kvp(ref mut kvp)) => {
                if let Some(s) = kvp.get_mut(&self.field) {
                    let old: i64 = (&*s).try_into()?;
                    let new = old + self.value;
                    *s = SimpleType::Integer(new);
                    Ok(new)
                } else {
                    kvp.insert_mut(self.field, SimpleType::Integer(self.value));
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

    use parking_lot::RwLock;

    use crate::{
        slot::{cmd::kvp::*, dict::Dict, Read, Write},
        utils::options::NxXx,
    };

    #[test]
    fn test1() {
        let dict = RwLock::new(Dict::new());
        let res = set::Req {
            key: b"hello"[..].into(),
            entries: vec![("k1".into(), "1".into()), ("k2".into(), "2".into())],
            nx_xx: NxXx::None,
        }
        .apply(1, dict.write().borrow_mut())
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
        .apply(1, dict.write().borrow_mut())
        .unwrap();
        assert_eq!(res, 2);

        let res = incr::Req {
            key: b"hello"[..].into(),
            field: "k3".into(),
            value: 10,
        }
        .apply(1, dict.write().borrow_mut())
        .unwrap();
        assert_eq!(res, 10);

        let res = get_all::Req {
            key: b"hello"[..].into(),
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
                    ("k1".into(), 2.into()),
                    ("k2".into(), "2".into()),
                    ("k3".into(), 10.into()),
                ];
                v.sort_unstable();
                v
            }
        );
    }
}
