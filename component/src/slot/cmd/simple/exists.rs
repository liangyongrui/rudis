use parking_lot::RwLock;

use crate::slot::{cmd::Read, data_type::KeyType, dict::Dict};

#[derive(Debug, Clone)]
pub struct Req<'a> {
    pub key: &'a KeyType,
}

impl<'a> Read<bool> for Req<'a> {
    fn apply(self, dict: &RwLock<Dict>) -> crate::Result<bool> {
        Ok(dict.read().d_exists(self.key))
    }
}

#[cfg(test)]
mod test {
    use std::borrow::BorrowMut;

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
        let res = exists::Req {
            key: &"hello".into(),
        }
        .apply(&dict)
        .unwrap();
        assert!(!res);
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
        let res = exists::Req {
            key: &"hello".into(),
        }
        .apply(&dict)
        .unwrap();
        assert!(res);
    }
}
