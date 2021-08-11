use parking_lot::RwLock;

use crate::slot::{cmd::Read, data_type::DataType, dict::Dict};

#[derive(Debug, Clone)]
pub struct Req<'a> {
    pub key: &'a [u8],
    pub member: &'a str,
    /// true 大的在前， false 小的在前
    pub rev: bool,
}

impl Read<Option<usize>> for Req<'_> {
    #[tracing::instrument(skip(dict), level = "debug")]
    fn apply(self, dict: &RwLock<Dict>) -> crate::Result<Option<usize>> {
        dict.read().d_get(self.key).map_or(Ok(None), |v| {
            if let DataType::SortedSet(ref sorted_set) = v.data {
                if sorted_set.hash.contains_key(self.member) {
                    let mut ans = 0;
                    if self.rev {
                        for n in sorted_set.value.iter().rev() {
                            ans += 1;
                            if n.key == self.member {
                                return Ok(Some(ans - 1));
                            }
                        }
                    } else {
                        for n in sorted_set.value.iter() {
                            ans += 1;
                            if n.key == self.member {
                                return Ok(Some(ans - 1));
                            }
                        }
                    }
                }
                Ok(None)
            } else {
                Err("error type".into())
            }
        })
    }
}
