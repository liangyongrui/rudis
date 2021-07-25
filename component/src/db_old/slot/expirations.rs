use std::{collections::BTreeMap, sync::Arc};

use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use tokio::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Notify,
    },
    time,
};
use tracing::debug;

use crate::db::{data_type::SimpleType, dict::Dict};

pub struct ExpirationEntry {
    pub id: u64,
    pub key: SimpleType,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct Expiration {
    pub sender: Sender<ExpirationEntry>,
    pub data: Arc<Mutex<Data>>,
    pub notify: Arc<Notify>,
}

#[derive(Debug)]
pub struct Data {
    pub expirations: BTreeMap<(DateTime<Utc>, u64), SimpleType>,
    shutdown: bool,
}

impl Data {
    fn new() -> Self {
        Self {
            expirations: BTreeMap::new(),
            shutdown: false,
        }
    }
}

impl Expiration {
    pub fn new(entry: Dict) -> Self {
        let notify = Arc::new(Notify::new());
        let data = Arc::new(Mutex::new(Data::new()));
        let (sender, receiver) = mpsc::channel(100);
        tokio::spawn(Expiration::receiver_listener(
            Arc::clone(&notify),
            Arc::clone(&data),
            receiver,
        ));
        tokio::spawn(Expiration::purge_expired_tasks(
            Arc::clone(&notify),
            Arc::clone(&data),
            entry,
        ));
        Self {
            sender,
            data,
            notify,
        }
    }

    fn is_shutdown(data: &Mutex<Data>) -> bool {
        data.lock().shutdown
    }
    async fn receiver_listener(
        notify: Arc<Notify>,
        data: Arc<Mutex<Data>>,
        mut receiver: Receiver<ExpirationEntry>,
    ) {
        while let Some(ExpirationEntry {
            id,
            key,
            expires_at,
        }) = receiver.recv().await
        {
            debug!(id, ?key, ?expires_at);
            let mut data = data.lock();
            let need_notify = data
                .expirations
                .keys()
                .next()
                .map(|expiration| expiration.0 > expires_at)
                .unwrap_or(true);
            data.expirations.insert((expires_at, id), key);
            drop(data);
            if need_notify {
                notify.notify_one();
            }
        }
        data.lock().shutdown = true;
    }

    async fn purge_expired_tasks(notify: Arc<Notify>, data: Arc<Mutex<Data>>, entry: Dict) {
        while !Expiration::is_shutdown(&data) {
            if let Some(when) = Expiration::purge_expired_keys(&data, &entry) {
                tokio::select! {
                    _ = time::sleep((when - Utc::now()).to_std().unwrap()) =>{}
                    _ = notify.notified() => {}
                }
            } else {
                // There are no keys expiring in the future. Wait until the task is
                // notified.
                notify.notified().await;
            }
        }
    }

    fn purge_expired_keys(data: &Mutex<Data>, entry: &Dict) -> Option<DateTime<Utc>> {
        let now = Utc::now();
        let mut data = data.lock();
        // 因为只需要处理头部元素，所有这里每次产生一个新的迭代器是安全的, 等first_entry stable 可以替换
        while let Some((&(when, id), key)) = data.expirations.iter().next() {
            if when > now {
                return Some(when);
            }
            let need_remove = entry.process_mut(key, |e| match e {
                Some(e) => e.id == id,
                None => false,
            });

            if need_remove {
                entry.remove(key);
                debug!("purge_expired_keys: {:?}", key);
            }
            data.expirations.remove(&(when, id));
        }
        None
    }
}
