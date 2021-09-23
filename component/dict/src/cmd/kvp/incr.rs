use std::convert::TryInto;

use keys::Key;
use serde::{Deserialize, Serialize};

use crate::{
    cmd::{Write, WriteCmd},
    data_type::{DataType, Kvp},
    Dict, Value,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Req {
    pub key: Key,
    pub field: Box<[u8]>,
    pub value: i64,
}
impl From<Req> for WriteCmd {
    fn from(req: Req) -> Self {
        Self::KvpIncr(req)
    }
}

/// 返回 更新后的值
impl<D: Dict> Write<i64, D> for Req {
    #[tracing::instrument(skip(dict), level = "debug")]
    fn apply(self, dict: &mut D) -> common::Result<i64> {
        let v = dict.get_or_insert_with(self.key, || Value {
            expires_at: 0,
            data: DataType::Kvp(Box::new(Kvp::new())),
            last_visit_time: common::now_timestamp_ms(),
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
            _ => Err("WRONGTYPE Operation against a key holding the wrong kind of value".into()),
        }
    }
}

#[cfg(test)]
mod test {
    use common::options::NxXx;

    use crate::{
        cmd::{
            kvp::{get_all, incr, set},
            Read, Write,
        },
        MemDict,
    };

    #[test]
    fn test1() {
        let mut dict = MemDict::default();
        let res = set::Req {
            key: b"hello"[..].into(),
            entries: vec![
                (b"k1"[..].into(), b"1"[..].into()),
                (b"k2"[..].into(), b"2"[..].into()),
            ],
            nx_xx: NxXx::None,
        }
        .apply(&mut dict)
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
            field: b"k1"[..].into(),
            value: 1,
        }
        .apply(&mut dict)
        .unwrap();
        assert_eq!(res, 2);

        let res = incr::Req {
            key: b"hello"[..].into(),
            field: b"k3"[..].into(),
            value: 10,
        }
        .apply(&mut dict)
        .unwrap();
        assert_eq!(res, 10);

        let res = get_all::Req {
            key: b"hello"[..].into(),
        }
        .apply(&mut dict)
        .unwrap();
        assert_eq!(
            {
                let mut v = res.into_iter().map(|kv| (kv.0, kv.1)).collect::<Vec<_>>();
                v.sort_unstable_by_key(|t| t.0.clone());
                v
            },
            vec![
                (b"k1"[..].into(), 2.into()),
                (b"k2"[..].into(), b"2"[..].into()),
                (b"k3"[..].into(), 10.into())
            ]
        );
    }
}
