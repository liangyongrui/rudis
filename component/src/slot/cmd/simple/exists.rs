use parking_lot::RwLock;

use crate::slot::{cmd::Read, dict::Dict};

#[derive(Debug, Clone)]
pub struct Req<'a> {
    pub key: &'a [u8],
}

impl<'a> Read<bool> for Req<'a> {
    #[tracing::instrument(skip(dict), level = "debug")]
    fn apply(self, dict: &RwLock<Dict>) -> crate::Result<bool> {
        Ok(dict.read().d_exists(self.key))
    }
}

#[cfg(test)]
mod test {
    use std::borrow::BorrowMut;

    use parking_lot::RwLock;

    use crate::{
        slot::{
            cmd::{simple::*, ExpiresStatus, ExpiresStatusUpdate, ExpiresWriteResp},
            data_type::DataType,
            dict::Dict,
            ExpiresWrite, Read,
        },
        utils::options::{ExpiresAt, NxXx},
    };

    #[test]
    fn test1() {
        let dict = RwLock::new(Dict::new());
        let res = exists::Req {
            key: b"hello"[..].into(),
        }
        .apply(&dict)
        .unwrap();
        assert!(!res);
        let date_time = crate::utils::now_timestamp_ms() + 1000;
        let cmd = set::Req {
            key: b"hello"[..].into(),
            value: "world".into(),
            expires_at: ExpiresAt::Specific(date_time),
            nx_xx: NxXx::None,
        };
        let res = cmd.apply(dict.write().borrow_mut()).unwrap();
        assert_eq!(
            res,
            ExpiresWriteResp {
                payload: DataType::Null,
                expires_status: ExpiresStatus::Update(ExpiresStatusUpdate {
                    key: b"hello"[..].into(),
                    before: 0,
                    new: date_time
                })
            }
        );
        let res = exists::Req {
            key: b"hello"[..].into(),
        }
        .apply(&dict)
        .unwrap();
        assert!(res);
    }
}
