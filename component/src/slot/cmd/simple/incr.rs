use std::{convert::TryInto, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::slot::{
    cmd::{Write, WriteCmd},
    data_type::SimpleType,
    dict::{self, Dict},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Req {
    pub key: Arc<[u8]>,
    pub value: i64,
}
impl From<Req> for WriteCmd {
    fn from(req: Req) -> Self {
        Self::Incr(req)
    }
}

/// 返回 更新后的值
impl Write<i64> for Req {
    fn apply(self, id: u64, dict: &mut Dict) -> crate::Result<i64> {
        if let Some(v) = dict.d_get_mut(&self.key) {
            match v.data {
                crate::slot::data_type::DataType::SimpleType(ref mut s) => {
                    let old: i64 = (&*s).try_into()?;
                    let new = old + self.value;
                    *s = SimpleType::Integer(new);
                    Ok(new)
                }
                crate::slot::data_type::DataType::CollectionType(_) => Err("error type".into()),
            }
        } else {
            dict.insert(
                self.key,
                dict::Value {
                    expires_at: None,
                    id,
                    data: self.value.into(),
                },
            );
            Ok(self.value)
        }
    }
}

#[cfg(test)]
mod test {
    use std::borrow::BorrowMut;

    use parking_lot::RwLock;

    use crate::slot::{cmd::simple::*, dict::Dict, Write};

    #[test]
    fn test1() {
        let dict = RwLock::new(Dict::new());
        let cmd = incr::Req {
            key: b"hello"[..].into(),
            value: 10,
        };
        let res = cmd.apply(1, dict.write().borrow_mut()).unwrap();
        assert_eq!(res, 10);
        let cmd = incr::Req {
            key: b"hello"[..].into(),
            value: -5,
        };
        let res = cmd.apply(1, dict.write().borrow_mut()).unwrap();
        assert_eq!(res, 5);
    }
}
