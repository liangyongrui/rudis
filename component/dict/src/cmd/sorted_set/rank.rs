use crate::{cmd::Read, data_type::DataType, Dict};

#[derive(Debug, Clone)]
pub struct Req<'a> {
    pub key: &'a [u8],
    pub member: &'a [u8],
    /// true 大的在前， false 小的在前
    pub rev: bool,
}

impl<D: Dict> Read<Option<usize>, D> for Req<'_> {
    #[tracing::instrument(skip(dict), level = "debug")]
    fn apply(self, dict: &D) -> common::Result<Option<usize>> {
        dict.get(self.key).map_or(Ok(None), |v| {
            if let DataType::SortedSet(ref sorted_set) = v.data {
                if sorted_set.hash.contains_key(self.member) {
                    let mut ans = 0;
                    if self.rev {
                        for n in sorted_set.value.iter().rev() {
                            ans += 1;
                            if &*n.key == self.member {
                                return Ok(Some(ans - 1));
                            }
                        }
                    } else {
                        for n in &sorted_set.value {
                            ans += 1;
                            if &*n.key == self.member {
                                return Ok(Some(ans - 1));
                            }
                        }
                    }
                }
                Ok(None)
            } else {
                Err("WRONGTYPE Operation against a key holding the wrong kind of value".into())
            }
        })
    }
}
