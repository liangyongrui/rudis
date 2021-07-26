use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::slot::{
    cmd::{Write, WriteCmd, WriteResp},
    data_type::SimpleType,
    dict::Dict,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Req {
    pub key: SimpleType,
    pub expires_at: Option<DateTime<Utc>>,
}
impl From<Req> for WriteCmd {
    fn from(req: Req) -> Self {
        Self::Expire(req)
    }
}
/// 返回 是否更新成功
impl Write<bool> for Req {
    fn apply(self, id: u64, dict: &mut Dict) -> crate::Result<WriteResp<bool>> {
        if let Some(v) = dict.d_get_mut(&self.key) {
            v.id = id;
            v.expire_at = self.expires_at;
            Ok(WriteResp {
                payload: true,
                new_expires_at: self.expires_at.map(|ea| (ea, self.key)),
            })
        } else {
            Ok(WriteResp {
                payload: false,
                new_expires_at: None,
            })
        }
    }
}

#[cfg(test)]
mod test {
    use std::{borrow::BorrowMut, thread::sleep};

    use chrono::{Duration, Utc};
    use parking_lot::RwLock;

    use crate::{
        slot::{
            cmd::{simple::*, WriteResp},
            data_type::SimpleType,
            dict::Dict,
            Read, Write,
        },
        utils::options::{ExpiresAt, NxXx},
    };

    #[test]
    fn test1() {
        let dict = RwLock::new(Dict::new());
        let date_time = Utc::now() + Duration::seconds(1);
        let cmd = set::Req {
            key: "hello".into(),
            value: "world".into(),
            expires_at: ExpiresAt::None,
            nx_xx: NxXx::None,
        };
        let res = cmd.apply(1, dict.write().borrow_mut()).unwrap();
        assert_eq!(
            res,
            WriteResp {
                payload: SimpleType::Null,
                new_expires_at: None,
            }
        );
        let res = exists::Req {
            key: &"hello".into(),
        }
        .apply(&dict)
        .unwrap();
        assert!(res);

        let cmd = expire::Req {
            key: "hello".into(),
            expires_at: Some(date_time),
        };
        let res = cmd.apply(1, dict.write().borrow_mut()).unwrap();
        assert_eq!(
            res,
            WriteResp {
                payload: true,
                new_expires_at: Some((date_time, "hello".into()))
            }
        );
        let res = exists::Req {
            key: &"hello".into(),
        }
        .apply(&dict)
        .unwrap();
        assert!(res);

        sleep(Duration::seconds(1).to_std().unwrap());
        let res = exists::Req {
            key: &"hello".into(),
        }
        .apply(&dict)
        .unwrap();
        assert!(!res);
    }
}
