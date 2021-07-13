mod expirations;

use std::{
    convert::TryInto,
    sync::{atomic::AtomicU64, Arc},
};

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use tracing::debug;

use self::expirations::{Expiration, ExpirationEntry};
use super::{
    data_type::{DataType, SimpleType},
    result::Result,
};
use crate::utils::options::NxXx;

/// Entry in the key-value store
#[derive(Debug)]
pub struct Entry {
    /// Uniquely identifies this entry.
    pub id: u64,

    /// Stored data
    pub data: DataType,

    /// Instant at which the entry expires and should be removed from the
    /// database.
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct Slot {
    /// The key-value data. We are not trying to do anything fancy so a
    /// `std::collections::HashMap` works fine.
    pub entries: Arc<DashMap<SimpleType, Entry>>,

    /// Tracks key TTLs.
    expirations: Expiration,

    /// Identifier to use for the next expiration. Each expiration is associated
    /// with a unique identifier. See above for why.
    next_id: AtomicU64,
}

impl Slot {
    pub fn new() -> Self {
        let entries = Arc::new(DashMap::new());
        let expirations = Expiration::new(Arc::clone(&entries));
        Self {
            entries,
            expirations,
            next_id: AtomicU64::new(0),
        }
    }

    pub fn get_simple(&self, key: &SimpleType) -> Result<Option<SimpleType>> {
        match self.entries.get(key) {
            Some(s) => match s.value() {
                Entry {
                    data: DataType::SimpleType(st),
                    ..
                } => Ok(Some(st.clone())),
                _ => Err("类型错误".to_string()),
            },
            None => Ok(None),
        }
    }

    pub fn get_or_insert_entry(
        &self,
        key: &SimpleType,
        f: fn() -> (DataType, Option<DateTime<Utc>>),
    ) -> dashmap::mapref::one::RefMut<'_, SimpleType, Entry> {
        if !self.entries.contains_key(key) {
            let (data, expires_at) = f();
            let e = Entry {
                id: self.next_id(),
                data,
                expires_at,
            };
            self.entries.insert(key.to_owned(), e);
        }
        self.entries.get_mut(key).unwrap()
    }

    /// Get and increment the next insertion ID.
    pub fn next_id(&self) -> u64 {
        self.next_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }

    pub fn exists(&self, key: &SimpleType) -> bool {
        self.entries.contains_key(key)
    }

    pub fn get_expires_at(&self, key: &SimpleType) -> Option<DateTime<Utc>> {
        debug!(
            "key: {:?}, value: {:?}",
            key,
            self.entries.get(key).and_then(|t| t.expires_at)
        );
        self.entries.get(key).and_then(|t| t.expires_at)
    }

    pub fn set_expires_at(&self, key: SimpleType, new_time: DateTime<Utc>) -> bool {
        self.entries
            .get(&key)
            .and_then(|t| t.expires_at.map(|p| (p, t.id)))
            .map(|(pre_time, id)| self.expirations.update(id, pre_time, new_time))
            .unwrap_or(false)
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
        let prev = self.entries.insert(
            key,
            Entry {
                id,
                data: value.into(),
                expires_at,
            },
        );
        prev.map(|t| t.data.try_into().unwrap())
    }

    pub fn remove(&self, key: &SimpleType) -> Option<DataType> {
        self.entries.remove(key).map(|prev| prev.1.data)
    }

    pub(crate) async fn set(
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
    use tracing::Level;

    use super::Slot;
    use crate::{utils::options::NxXx, SimpleType};

    // #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    #[tokio::test]
    async fn test_set() {
        tracing_subscriber::fmt::Subscriber::builder()
            .with_max_level(Level::DEBUG)
            .try_init()
            .unwrap();
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
        sleep(Duration::from_secs(1)).await;
        assert_eq!(slot.get_expires_at(&key), None);
    }
}
