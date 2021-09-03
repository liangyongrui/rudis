use crate::{cmd::Read, Dict};

#[derive(Debug, Clone)]
pub struct Req<'a> {
    pub key: &'a [u8],
}

impl<'a> Read<bool> for Req<'a> {
    #[tracing::instrument(skip(dict), level = "debug")]
    fn apply(self, dict: &Dict) -> common::Result<bool> {
        Ok(dict.exists(self.key))
    }
}

#[cfg(test)]
mod test {
    use common::{
        now_timestamp_ms,
        options::{ExpiresAt, NxXx},
    };

    use crate::{
        cmd::{
            simple::{exists, set},
            ExpiresStatus, ExpiresStatusUpdate, ExpiresWrite, ExpiresWriteResp, Read,
        },
        data_type::DataType,
        Dict,
    };

    #[test]
    fn test1() {
        let mut dict = Dict::default();
        let res = exists::Req {
            key: b"hello"[..].into(),
        }
        .apply(&dict)
        .unwrap();
        assert!(!res);
        let date_time = now_timestamp_ms() + 1000;
        let cmd = set::Req {
            key: b"hello"[..].into(),
            value: "world".into(),
            expires_at: ExpiresAt::Specific(date_time),
            nx_xx: NxXx::None,
        };
        let res = cmd.apply(&mut dict).unwrap();
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
