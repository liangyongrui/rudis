//! 自动过期处理task
//!
//! 异步删除key，减少持有锁的时间

use std::{borrow::Borrow, collections::BTreeSet, sync::Arc};

use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use tokio::{sync::Notify, time};
use tracing::debug;

use crate::db::Db;

/// When derived on structs, it will produce a lexicographic ordering
/// based on the top-to-bottom declaration order of the struct’s members.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Entry {
    pub expires_at: DateTime<Utc>,
    pub slot: u16,
    pub id: u64,
    pub key: Vec<u8>,
}

pub enum Message {
    /// 清空指定slot
    /// 用于移动slot，替换dict
    Clear(u16),
    Add(Entry),
}

#[derive(Debug)]
pub struct Expiration {
    data: Arc<Mutex<BTreeSet<Entry>>>,
    notify: Arc<Notify>,
    pub tx: flume::Sender<Message>,
    rx: flume::Receiver<Message>,
}

impl Expiration {
    pub fn new() -> Self {
        let (tx, rx) = flume::unbounded();

        Self {
            tx,
            rx,
            data: Arc::new(Mutex::new(BTreeSet::new())),
            notify: Arc::new(Notify::new()),
        }
    }

    pub fn listen(self, db: Arc<Db>) -> flume::Sender<Message> {
        let Expiration {
            data,
            notify,
            tx,
            rx,
        } = self;

        let data_c = Arc::clone(&data);
        let notify_c = Arc::clone(&notify);
        tokio::spawn(Expiration::recv_task(data_c, notify_c, rx));
        tokio::spawn(Expiration::purge_expired_task(data, notify, db));
        tx
    }

    async fn recv_task(
        data: Arc<Mutex<BTreeSet<Entry>>>,
        notify: Arc<Notify>,
        rx: flume::Receiver<Message>,
    ) {
        while let Ok(e) = rx.recv_async().await {
            match e {
                Message::Clear(slot) => {
                    data.lock().retain(|e| e.slot != slot);
                }
                Message::Add(e) => {
                    let mut lock = data.lock();
                    let need_notify = lock
                        .iter()
                        .next()
                        .map(|ne| ne.expires_at > e.expires_at)
                        .unwrap_or(true);
                    lock.insert(e);
                    drop(lock);
                    if need_notify {
                        notify.notify_one();
                    }
                }
            }
        }
    }

    async fn purge_expired_task(
        data: Arc<Mutex<BTreeSet<Entry>>>,
        notify: Arc<Notify>,
        db: Arc<Db>,
    ) {
        loop {
            let next = Expiration::purge_expired_keys(&data, db.borrow());
            if let Some(when) = next {
                tokio::select! {
                    _ = time::sleep((when - Utc::now()).to_std().unwrap()) =>{}
                    _ = notify.notified() => {}
                }
            } else {
                // There are no keys expiring in the future.
                // Wait until the task is notified.
                notify.notified().await;
            }
        }
    }

    fn purge_expired_keys(data: &Mutex<BTreeSet<Entry>>, db: &Db) -> Option<DateTime<Utc>> {
        let now = Utc::now();
        loop {
            // 减少持有锁的时间
            let entry = {
                let mut btree_lock = data.lock();
                //  等 #![feature(map_first_last)] stable 可以替换
                let entry = match btree_lock.iter().next() {
                    Some(e) => e.clone(),
                    None => return None,
                };
                btree_lock.remove(&entry);
                entry
            };

            let expires_at = entry.expires_at;
            if expires_at > now {
                return Some(expires_at);
            }

            let slot = db.get_slot_by_id(&entry.slot);
            // 取出数据之后再析构，避免持有过长时间的slot锁
            let expired_data = {
                let mut lock = slot.dict.write();
                match lock.get(&entry.key) {
                    Some(value) if value.id == entry.id => Some(lock.remove(&entry.key)),
                    _ => None,
                }
            };

            if expired_data.is_some() {
                debug!("purge expired: {:?}", entry);
            } else {
                debug!("purge covered: {:?}", entry);
            }
        }
    }
}
