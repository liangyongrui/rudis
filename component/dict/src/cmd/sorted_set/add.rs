use common::options::{GtLt, NxXx};
use keys::Key;
use serde::{Deserialize, Serialize};

use crate::{
    cmd::{Write, WriteCmd},
    data_type::{self, DataType, SortedSet},
    Dict, Value,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Req {
    pub key: Key,
    pub nodes: Vec<data_type::sorted_set::Node>,
    /// - XX: Only update elements that already exist. Don't add new elements.
    /// - NX: Only add new elements. Don't update already existing elements.
    pub nx_xx: NxXx,
    /// - LT: Only update existing elements if the new score is less than the current score.
    /// This flag doesn't prevent adding new elements.
    /// - GT: Only update existing elements if the new score is greater than the current score.
    /// This flag doesn't prevent adding new elements.
    pub gt_lt: GtLt,
    pub incr: bool,
}

pub struct Resp {
    /// 原来的大小
    pub old_len: usize,
    /// 更新后大小
    pub new_len: usize,
    /// 更新成功的node数
    pub update_len: usize,
}
impl From<Req> for WriteCmd {
    #[inline]
    fn from(req: Req) -> Self {
        Self::SortedSetAdd(req)
    }
}
impl<D: Dict> Write<Resp, D> for Req {
    #[tracing::instrument(skip(dict), level = "debug")]
    fn apply(self, dict: &mut D) -> common::Result<Resp> {
        let old = dict.get_or_insert_with(self.key, || Value {
            data: DataType::SortedSet(Box::new(SortedSet::new())),
            expires_at: 0,
            visit_log: Value::new_visit_log(),
        });
        if let DataType::SortedSet(ref mut sorted_set) = old.data {
            let old_len = sorted_set.hash.len();
            let mut update_len = 0;
            for mut node in self.nodes {
                let can_update = match (self.nx_xx, self.gt_lt) {
                    (NxXx::Nx, GtLt::None) => !sorted_set.hash.contains_key(&node.key),
                    (NxXx::Nx, _) => false,
                    (_, GtLt::Gt) => sorted_set
                        .hash
                        .get(&node.key)
                        .filter(|x| x.score < node.score)
                        .is_some(),
                    (_, GtLt::Lt) => sorted_set
                        .hash
                        .get(&node.key)
                        .filter(|x| x.score > node.score)
                        .is_some(),
                    (NxXx::Xx, GtLt::None) => sorted_set.hash.contains_key(&node.key),
                    (NxXx::None, GtLt::None) => true,
                };
                if can_update {
                    update_len += 1;
                    if let Some(on) = sorted_set.hash.insert(node.key.clone(), node.clone()) {
                        sorted_set.value.remove(&on);
                        if self.incr {
                            node.score.0 += on.score.0;
                        }
                    }
                    sorted_set.value.insert(node);
                }
            }
            Ok(Resp {
                old_len,
                update_len,
                new_len: sorted_set.hash.len(),
            })
        } else {
            Err("WRONGTYPE Operation against a key holding the wrong kind of value".into())
        }
    }
}
