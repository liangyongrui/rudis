//! 自动过期处理task
//!
//! 异步删除key，减少持有锁的时间

use std::{borrow::Borrow, collections::BTreeSet, sync::Arc, time::Duration};

use parking_lot::Mutex;
use tokio::{sync::Notify, time};
use tracing::debug;

use crate::{db::Db, slot::cmd::ExpiresStatusUpdate, utils::now_timestamp_ms};

/// When derived on structs, it will produce a lexicographic ordering
/// based on the top-to-bottom declaration order of the struct’s members.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Entry {
    pub expires_at: u64,
    pub slot: u16,
    pub key: Arc<[u8]>,
}

pub enum Message {
    /// 清空指定slot
    /// 用于移动slot，替换dict
    Clear(u16),
    Update(Update),
    /// 批量插入
    BatchAdd(Vec<Entry>),
}

pub struct Update {
    pub status: ExpiresStatusUpdate,
    pub slot: u16,
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

    pub fn listen(self, db: Arc<Db>) {
        let Expiration {
            data, notify, rx, ..
        } = self;

        let data_c = Arc::clone(&data);
        let notify_c = Arc::clone(&notify);
        tokio::spawn(Expiration::recv_task(data_c, notify_c, rx));
        tokio::spawn(Expiration::purge_expired_task(data, notify, db));
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
                Message::Update(Update { slot, status }) => {
                    debug!(slot, ?status);
                    debug_assert_ne!(status.new, status.before);
                    let mut lock = data.lock();
                    let need_notify = if status.new > 0 {
                        let res = lock
                            .iter()
                            .next()
                            .map_or(true, |ne| ne.expires_at > status.new);
                        lock.insert(Entry {
                            expires_at: status.new,
                            slot,
                            key: status.key.clone(),
                        });
                        res
                    } else {
                        false
                    };
                    if status.before > 0 {
                        lock.remove(&Entry {
                            expires_at: status.before,
                            slot,
                            key: status.key,
                        });
                    }
                    drop(lock);
                    if need_notify {
                        notify.notify_one();
                    }
                }
                Message::BatchAdd(vs) => {
                    let mut lock = data.lock();
                    for v in vs {
                        lock.insert(v);
                    }
                    drop(lock);
                    notify.notify_one();
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
            debug!(next);
            if next == 0 {
                // There are no keys expiring in the future.
                // Wait until the task is notified.
                notify.notified().await;
                debug!("notify 2");
            }
            let now = now_timestamp_ms();
            if next > now {
                tokio::select! {
                    _ = time::sleep(Duration::from_millis(next - now)) =>{}
                    _ = notify.notified() => {
                        debug!("notify 1");
                    }
                }
            }
        }
    }

    fn purge_expired_keys(data: &Mutex<BTreeSet<Entry>>, db: &Db) -> u64 {
        let now = now_timestamp_ms();
        loop {
            // 减少持有锁的时间
            let entry = {
                let mut btree_lock = data.lock();
                //  等 #![feature(map_first_last)] stable 可以替换
                let entry = match btree_lock.iter().next() {
                    Some(e) => e.clone(),
                    None => return 0,
                };
                let expires_at = entry.expires_at;
                if expires_at > now {
                    return expires_at;
                }
                btree_lock.remove(&entry);
                entry
            };

            let slot = db.get_slot_by_id(&entry.slot);
            // 取出数据之后再析构，避免持有过长时间的slot锁
            let expired_data = {
                let mut lock = slot.dict.write();
                debug!("before: slot: {}, dict_len: {}", entry.slot, lock.len());
                let res = match lock.get(&entry.key) {
                    // 如果过期时间更新过，可能会有时间不一样的情况
                    Some(value) if value.expires_at == entry.expires_at => {
                        Some(lock.remove(&entry.key))
                    }
                    _ => None,
                };
                debug!("after: slot: {}, dict_len: {}", entry.slot, lock.len());
                res
            };

            if expired_data.is_some() {
                debug!("purge expired: {:?}", entry);
            } else {
                debug!("purge covered: {:?}", entry);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use tokio::time::sleep;

    use crate::{
        slot::cmd,
        utils::{now_timestamp_ms, options::ExpiresAt},
    };

    #[tokio::test]
    async fn test() {
        let _ = tracing_subscriber::fmt::Subscriber::builder()
            .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
            .with_max_level(tracing::Level::DEBUG)
            .try_init();

        let db = crate::db::Db::new().await;
        db.set(cmd::simple::set::Req {
            key: (&b"1"[..]).into(),
            value: "123".into(),
            expires_at: ExpiresAt::Specific(now_timestamp_ms() + 1000),
            nx_xx: crate::utils::options::NxXx::None,
        })
        .unwrap();
        sleep(Duration::from_secs(2)).await;
    }
}
