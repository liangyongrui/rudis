use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use tokio::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Notify,
    },
    time,
};
use tracing::debug;

use crate::db::DataType;

pub struct Expiration {
    sender: Sender<(DateTime<Utc>, u64, String)>,
    data: Arc<Mutex<Data>>,
}

pub struct Data {
    expirations: BTreeMap<(DateTime<Utc>, u64), String>,
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
    pub fn new(entry: Arc<DashMap<String, DataType>>) -> Self {
        let notify = Arc::new(Notify::new());
        let data = Arc::new(Mutex::new(Data::new()));
        let (sender, receiver) = mpsc::channel(100);
        tokio::spawn(Expiration::receiver_listener(
            Arc::clone(&notify),
            Arc::clone(&data),
            receiver,
        ));
        tokio::spawn(Expiration::purge_expired_tasks(
            notify,
            Arc::clone(&data),
            entry,
        ));
        Self { sender, data }
    }

    fn is_shutdown(data: &Mutex<Data>) -> bool {
        data.lock().unwrap().shutdown
    }
    async fn receiver_listener(
        notify: Arc<Notify>,
        data: Arc<Mutex<Data>>,
        mut receiver: Receiver<(DateTime<Utc>, u64, String)>,
    ) {
        while let Some((time, id, key)) = receiver.recv().await {
            let mut data = data.lock().unwrap();
            let need_notify = data
                .expirations
                .keys()
                .next()
                .map(|expiration| expiration.0 > time)
                .unwrap_or(true);
            data.expirations.insert((time, id), key);
            drop(data);
            if need_notify {
                notify.notify_one();
            }
        }
        data.lock().unwrap().shutdown = true;
    }

    async fn purge_expired_tasks(
        notify: Arc<Notify>,
        data: Arc<Mutex<Data>>,
        entry: Arc<DashMap<String, DataType>>,
    ) {
        while !Expiration::is_shutdown(&data) {
            if let Some(when) = Expiration::purge_expired_keys(&data, &entry).await {
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

    pub async fn purge_expired_keys(
        data: &Mutex<Data>,
        entry: &DashMap<String, DataType>,
    ) -> Option<DateTime<Utc>> {
        let now = Utc::now();
        let mut data = data.lock().unwrap();
        // 因为只需要处理头部元素，所有这里每次产生一个新的迭代器是安全的, 等first_entry stable 可以替换
        while let Some((&(when, id), key)) = data.expirations.iter().next() {
            if when > now {
                return Some(when);
            }
            debug!("purge_expired_keys: {}", key);
            entry.remove(key);
            data.expirations.remove(&(when, id));
        }
        None
    }
}
