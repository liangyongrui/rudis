use serde::{Deserialize, Serialize};

use crate::slot::{
    cmd::{Write, WriteCmd, WriteResp},
    data_type::{CollectionType, DataType, Set, SimpleType},
    dict::{self, Dict},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Req {
    pub key: SimpleType,
    pub members: Vec<SimpleType>,
}

pub struct Resp {
    /// 原来的大小
    pub old_len: usize,
    /// 更新后大小
    pub new_len: usize,
}
impl From<Req> for WriteCmd {
    fn from(req: Req) -> Self {
        Self::SetAdd(req)
    }
}
impl Write<Resp> for Req {
    fn apply(self, id: u64, dict: &mut Dict) -> crate::Result<WriteResp<Resp>> {
        let old = dict.d_get_mut_or_insert_with(self.key, || dict::Value {
            id,
            data: DataType::CollectionType(CollectionType::Set(Set::new())),
            expire_at: None,
        });
        if let DataType::CollectionType(CollectionType::Set(ref mut set)) = old.data {
            let old_len = set.size();
            for m in self.members {
                set.insert_mut(m);
            }
            Ok(WriteResp {
                new_expires_at: None,
                payload: Resp {
                    old_len,
                    new_len: set.size(),
                },
            })
        } else {
            Err("error type".into())
        }
    }
}

// todo utest
