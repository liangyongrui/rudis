use serde::{Deserialize, Serialize};

use crate::slot::{
    cmd::{Write, WriteCmd, WriteResp},
    dict::{Dict, Value},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Req {
    pub key: Vec<u8>,
}

impl From<Req> for WriteCmd {
    fn from(req: Req) -> Self {
        Self::Del(req)
    }
}
/// 返回 原始值
impl Write<Option<Value>> for Req {
    fn apply(self, _id: u64, dict: &mut Dict) -> crate::Result<WriteResp<Option<Value>>> {
        if dict.d_exists(&self.key) {
            Ok(WriteResp {
                payload: dict.remove(&self.key),
                new_expires_at: None,
            })
        } else {
            Ok(WriteResp {
                payload: None,
                new_expires_at: None,
            })
        }
    }
}

#[cfg(test)]
mod test {
    use std::borrow::BorrowMut;

    use chrono::{Duration, Utc};
    use parking_lot::RwLock;

    use super::*;
    use crate::{
        slot::{
            cmd::simple::*,
            data_type::{DataType, SimpleType},
            dict::Dict,
        },
        utils::options::{ExpiresAt, NxXx},
    };

    #[test]
    fn test1() {
        let dict = RwLock::new(Dict::new());
        let res = del::Req {
            key: "hello".into(),
        }
        .apply(2, dict.write().borrow_mut())
        .unwrap()
        .payload;
        assert_eq!(res, None);
        let date_time = Utc::now() + Duration::seconds(1);
        let cmd = set::Req {
            key: "hello".into(),
            value: "world".into(),
            expires_at: ExpiresAt::Specific(date_time),
            nx_xx: NxXx::None,
        };
        let res = cmd.apply(1, dict.write().borrow_mut()).unwrap();
        assert_eq!(
            res,
            WriteResp {
                payload: SimpleType::Null,
                new_expires_at: Some((date_time, "hello".into()))
            }
        );
        let res = del::Req {
            key: "hello".into(),
        }
        .apply(2, dict.write().borrow_mut())
        .unwrap()
        .payload
        .unwrap()
        .data;
        assert_eq!(res, DataType::SimpleType("world".into()));
    }
}
