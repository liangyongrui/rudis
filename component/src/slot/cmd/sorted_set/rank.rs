use std::borrow::Borrow;

use parking_lot::RwLock;
use rpds::RedBlackTreeSetSync;

use crate::slot::{
    cmd::Read,
    data_type::{sorted_set::Node, CollectionType, DataType, SimpleType},
    dict::Dict,
};

#[derive(Debug, Clone)]
pub struct Req<'a> {
    pub key: &'a [u8],
    pub member: &'a SimpleType,
    /// true 大的在前， false 小的在前
    pub rev: bool,
}

impl Read<Option<usize>> for Req<'_> {
    fn apply(self, dict: &RwLock<Dict>) -> crate::Result<Option<usize>> {
        if let Some(value) = self.apply_in_lock(dict.read().borrow())? {
            let mut ans = 0;
            if self.rev {
                for n in value.iter().rev() {
                    ans += 1;
                    if &n.key == self.member {
                        return Ok(Some(ans));
                    }
                }
            } else {
                for n in value.iter() {
                    ans += 1;
                    if &n.key == self.member {
                        return Ok(Some(ans));
                    }
                }
            }
        }
        Ok(None)
    }
}
impl Req<'_> {
    fn apply_in_lock(&self, dict: &Dict) -> crate::Result<Option<RedBlackTreeSetSync<Node>>> {
        if let Some(v) = dict.d_get(self.key) {
            if let DataType::CollectionType(CollectionType::SortedSet(ref sorted_set)) = v.data {
                if !sorted_set.hash.contains_key(self.member) {
                    Ok(None)
                } else {
                    Ok(Some(sorted_set.value.clone()))
                }
            } else {
                Err("error type".into())
            }
        } else {
            Ok(None)
        }
    }
}

// todo utest
