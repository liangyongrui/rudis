use std::{hash::Hash, sync::Arc};

use rpds::{HashTrieMapSync, HashTrieSetSync, RedBlackTreeSetSync};
use serde::Serialize;

use super::pointer::Bor;

impl<K: Serialize, V: Serialize> Serialize for Bor<Arc<HashTrieMapSync<K, V>>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        todo!()
    }
}

impl<V: Serialize + Eq + Hash> Serialize for Bor<Arc<HashTrieSetSync<V>>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        todo!()
    }
}
impl<V: Serialize + Ord> Serialize for Bor<Arc<RedBlackTreeSetSync<V>>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        todo!()
    }
}
