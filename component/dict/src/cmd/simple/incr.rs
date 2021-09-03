use std::{convert::TryInto, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::{
    cmd::{Write, WriteCmd},
    data_type::DataType,
    Dict, Value,
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
    #[tracing::instrument(skip(dict), level = "debug")]
    fn apply(self, dict: &mut Dict) -> common::Result<i64> {
        if let Some(v) = dict.get_mut(&self.key) {
            let old: i64 = (&v.data).try_into()?;
            let new = old + self.value;
            v.data = DataType::Integer(new);
            Ok(new)
        } else {
            dict.insert(
                self.key,
                Value {
                    expires_at: 0,
                    data: self.value.into(),
                },
            );
            Ok(self.value)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        cmd::{simple::incr, Write},
        Dict,
    };

    #[test]
    fn test1() {
        let mut dict = Dict::default();
        let cmd = incr::Req {
            key: b"hello"[..].into(),
            value: 10,
        };
        let res = cmd.apply(&mut dict).unwrap();
        assert_eq!(res, 10);
        let cmd = incr::Req {
            key: b"hello"[..].into(),
            value: -5,
        };
        let res = cmd.apply(&mut dict).unwrap();
        assert_eq!(res, 5);
    }
}
