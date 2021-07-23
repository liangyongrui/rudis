use std::{
    borrow::{Borrow, BorrowMut},
    collections::BTreeSet,
    sync::Arc,
};

use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use tokio::{
    sync::{mpsc, Notify},
    time,
};

use crate::{db2::Db, slot::data_type::SimpleType};

/// When derived on structs, it will produce a lexicographic ordering
/// based on the top-to-bottom declaration order of the struct’s members.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Entry {
    pub expires_at: DateTime<Utc>,
    pub slot: u16,
    pub id: u64,
    pub key: SimpleType,
}

#[derive(Debug)]
pub struct Expiration {
    data: Arc<Mutex<BTreeSet<Entry>>>,
    notify: Arc<Notify>,
    pub tx: mpsc::Sender<Entry>,
    rx: mpsc::Receiver<Entry>,
}

impl Expiration {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(1024);

        Self {
            tx,
            rx,
            data: Arc::new(Mutex::new(BTreeSet::new())),
            notify: Arc::new(Notify::new()),
        }
    }

    pub fn listen(self, db: Arc<Db>) -> mpsc::Sender<Entry> {
        let Expiration {
            data,
            notify,
            tx,
            rx,
        } = self;

        tokio::spawn(Expiration::recv_task(
            Arc::clone(&data),
            Arc::clone(&notify),
            rx,
        ));
        tokio::spawn(Expiration::purge_expired_task(data, notify, db));
        tx
    }

    async fn recv_task(
        data: Arc<Mutex<BTreeSet<Entry>>>,
        notify: Arc<Notify>,
        mut rx: mpsc::Receiver<Entry>,
    ) {
        while let Some(e) = rx.recv().await {
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

    async fn purge_expired_task(
        data: Arc<Mutex<BTreeSet<Entry>>>,
        notify: Arc<Notify>,
        db: Arc<Db>,
    ) {
        loop {
            let next = Expiration::purge_expired_keys(data.lock().borrow_mut(), db.borrow());
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

    fn purge_expired_keys(data: &mut BTreeSet<Entry>, _db: &Db) -> Option<DateTime<Utc>> {
        let now = Utc::now();
        // 因为只需要处理头部元素，所有这里每次产生一个新的迭代器是安全的, 等 #![feature(map_first_last)] stable 可以替换
        while let Some(Entry {
            expires_at,
            slot: _,
            id: _,
            key: _,
        }) = data.iter().next()
        {
            let expires_at = *expires_at;
            if expires_at > now {
                return Some(expires_at);
            }
            // TODO 判断是否需要删除，需要则删除

            // 新线程析构
            // let need_remove = entry.process_mut(key, |e| match e {
            //     Some(e) => e.id == id,
            //     None => false,
            // });

            // if need_remove {
            //     entry.remove(key);
            //     debug!("purge_expired_keys: {:?}", key);
            // }
            // data.expirations.remove(&(when, id));
        }
        None
    }
}
