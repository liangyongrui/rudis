pub mod dict;
mod expirations;

use std::{convert::TryInto, sync::atomic::AtomicU64};

use chrono::{DateTime, Utc};
use serde::{de::Visitor, Deserialize, Serialize};

use self::{
    dict::Dict,
    expirations::{Expiration, ExpirationEntry},
};
use super::{
    data_type::{DataType, SimpleType},
    result::Result,
};
use crate::utils::options::NxXx;

pub struct Slot {
    /// The key-value data. We are not trying to do anything fancy so a
    /// `std::collections::HashMap` works fine.
    pub dict: dict::Dict,

    /// Tracks key TTLs.
    expirations: Expiration,

    /// Identifier to use for the next expiration. Each expiration is associated
    /// with a unique identifier. See above for why.
    next_id: AtomicU64,
}

impl Serialize for Slot {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // let mut state = serializer.serialize_map(Some(self.dict.len()))?;
        // for pair in self.dict.iter() {
        //     state.serialize_entry(pair.key(), pair.value())?;
        // }
        // state.end()
        todo!()
    }
}

struct SlotVistor;

impl<'de> Visitor<'de> for SlotVistor {
    type Value = Slot;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Slot need a map")
    }

    fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        todo!()
        // let mut slot = Slot::new();
        // let mut big = 0;
        // let mut expirations = slot.expirations.data.lock().unwrap();
        // while let Some((key, value)) = map.next_entry::<SimpleType, dict::Entry>()? {
        //     big = big.max(value.id);
        //     if let Some(ea) = value.expires_at {
        //         expirations.expirations.insert((ea, value.id), key.clone());
        //     }
        //     slot.dict.insert(key, value);
        // }
        // drop(expirations);
        // slot.expirations.notify.notify_one();
        // slot.next_id = AtomicU64::new(big + 1);
        // Ok(slot)
    }
}
impl<'de> Deserialize<'de> for Slot {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_i32(SlotVistor)
    }
}

impl Slot {
    pub fn new() -> Self {
        let dict = Dict::new();
        let expirations = Expiration::new(dict.clone());
        Self {
            dict,
            expirations,
            next_id: AtomicU64::new(0),
        }
    }

    pub async fn get_or_insert<F: FnOnce(&mut dict::Entry) -> T, T>(
        &self,
        key: SimpleType,
        f: fn() -> (DataType, Option<DateTime<Utc>>),
        then_do: F,
    ) -> T {
        match self.dict.get_or_insert(key, f, then_do) {
            (res, None) => res,
            (res, Some(e)) => {
                let _ = self.expirations.sender.send(e).await;
                res
            }
        }
    }
    pub fn get_simple(&self, key: &SimpleType) -> Result<Option<SimpleType>> {
        self.dict.process_mut(key, |entry| match entry {
            Some(s) => match s {
                dict::Entry {
                    data: DataType::SimpleType(st),
                    ..
                } => Ok(Some(st.clone())),
                _ => Err("类型错误".to_string()),
            },
            None => Ok(None),
        })
    }

    /// Get and increment the next insertion ID.
    pub fn next_id(&self) -> u64 {
        //todo 删除
        0
        // self.next_id
        //     .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }

    pub fn exists(&self, key: &SimpleType) -> bool {
        self.dict.exists(key)
    }

    pub fn get_expires_at(&self, key: &SimpleType) -> Option<DateTime<Utc>> {
        self.dict.get_expires_at(key)
    }

    pub async fn set_expires_at(&self, key: &SimpleType, new_time: DateTime<Utc>) -> bool {
        let (id, pre_time) = if let Some(t) = self.dict.process_mut(key, |entry| match entry {
            Some(t) => Some((t.id, t.expires_at)),
            None => None,
        }) {
            (t.0, t.1)
        } else {
            return false;
        };

        if let Some(pre_time) = pre_time {
            self.expirations.update(id, pre_time, new_time);
        } else {
            let _ = self
                .expirations
                .sender
                .send(ExpirationEntry {
                    id,
                    key: key.clone(),
                    expires_at: new_time,
                })
                .await;
        }
        true
    }

    /// 调用之前需要自己保证原始值的value 为 simpleType 或 不存在
    async fn update_simple(
        &self,
        key: SimpleType,
        value: SimpleType,
        expires_at: Option<DateTime<Utc>>,
    ) -> Option<SimpleType> {
        let id = self.next_id();
        if let Some(expires_at) = expires_at {
            let _ = self
                .expirations
                .sender
                .send(ExpirationEntry {
                    id,
                    key: key.clone(),
                    expires_at,
                })
                .await;
        }
        let prev = self.dict.insert(
            key,
            dict::Entry {
                id,
                data: value.into(),
                expires_at,
            },
        );
        prev.map(|t| t.data.try_into().unwrap())
    }

    pub fn remove(&self, key: &SimpleType) -> Option<DataType> {
        self.dict.remove(key).map(|prev| prev.data)
    }

    pub async fn set(
        &self,
        key: SimpleType,
        value: SimpleType,
        nx_xx: NxXx,
        mut expires_at: Option<DateTime<Utc>>,
        keepttl: bool,
    ) -> Result<Option<SimpleType>> {
        let old_value = self.get_simple(&key)?;
        let need_update = match nx_xx {
            NxXx::Nx => old_value.is_none(),
            NxXx::Xx => old_value.is_some(),
            NxXx::None => true,
        };
        if !need_update {
            return Ok(old_value);
        }
        if keepttl {
            expires_at = self.get_expires_at(&key);
        }
        Ok(self.update_simple(key, value, expires_at).await)
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use chrono::Utc;
    use tokio::time::sleep;

    use super::Slot;
    use crate::{db::DataType, utils::options::NxXx, SimpleType};

    // #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    #[tokio::test]
    async fn test_set() {
        let slot = Slot::new();
        let key = SimpleType::SimpleString("123".to_owned());
        let res = slot
            .set(
                key.clone(),
                SimpleType::SimpleString("456".to_owned()),
                NxXx::None,
                None,
                false,
            )
            .await
            .unwrap();
        assert_eq!(res, None);
        assert_eq!(
            slot.get_simple(&key).unwrap(),
            Some(SimpleType::SimpleString("456".to_owned()))
        );
        let res = slot
            .set(
                key.clone(),
                SimpleType::SimpleString("4567".to_owned()),
                NxXx::Nx,
                None,
                false,
            )
            .await
            .unwrap();
        assert_eq!(res, Some(SimpleType::SimpleString("456".to_owned())));
        assert_eq!(
            slot.get_simple(&key).unwrap(),
            Some(SimpleType::SimpleString("456".to_owned()))
        );
        slot.set(
            key.clone(),
            SimpleType::SimpleString("4567".to_owned()),
            NxXx::None,
            None,
            false,
        )
        .await
        .unwrap();
        assert_eq!(
            slot.get_simple(&key).unwrap(),
            Some(SimpleType::SimpleString("4567".to_owned()))
        );
        let key2 = SimpleType::SimpleString("1234".to_owned());
        slot.set(
            key2.clone(),
            SimpleType::SimpleString("4567".to_owned()),
            NxXx::Xx,
            None,
            false,
        )
        .await
        .unwrap();
        assert_eq!(slot.get_simple(&key2).unwrap(), None);
        let ea = Some(Utc::now() + chrono::Duration::seconds(1));
        slot.set(
            key.clone(),
            SimpleType::SimpleString("4567".to_owned()),
            NxXx::None,
            ea,
            false,
        )
        .await
        .unwrap();
        assert_eq!(slot.get_expires_at(&key), ea);
        slot.set(
            key.clone(),
            SimpleType::SimpleString("45678".to_owned()),
            NxXx::None,
            None,
            true,
        )
        .await
        .unwrap();
        assert_eq!(slot.get_expires_at(&key), ea);
        sleep(Duration::from_secs(2)).await;
        assert_eq!(slot.get_expires_at(&key), None);
    }

    #[tokio::test]
    async fn test_remove() {
        let slot = Slot::new();
        let key = SimpleType::SimpleString("123".to_owned());
        let res = slot
            .set(
                key.clone(),
                SimpleType::SimpleString("456".to_owned()),
                NxXx::None,
                None,
                false,
            )
            .await
            .unwrap();
        assert_eq!(res, None);
        assert_eq!(
            slot.get_simple(&key).unwrap(),
            Some(SimpleType::SimpleString("456".to_owned()))
        );
        assert_eq!(
            slot.remove(&key),
            Some(DataType::SimpleType(SimpleType::SimpleString(
                "456".to_owned()
            )))
        );
        assert_eq!(slot.get_simple(&key).unwrap(), None)
    }

    #[tokio::test]
    async fn test_set_expires_at() {
        let slot = Slot::new();
        let key = SimpleType::SimpleString("123".to_owned());
        let res = slot
            .set(
                key.clone(),
                SimpleType::SimpleString("456".to_owned()),
                NxXx::None,
                None,
                false,
            )
            .await
            .unwrap();
        assert_eq!(res, None);
        assert_eq!(
            slot.get_simple(&key).unwrap(),
            Some(SimpleType::SimpleString("456".to_owned()))
        );
        let ea = Utc::now() + chrono::Duration::seconds(1);
        assert!(slot.set_expires_at(&key, ea).await);
        sleep(Duration::from_secs(2)).await;
        assert_eq!(slot.get_simple(&key).unwrap(), None)
    }
}
