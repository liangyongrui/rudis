use std::{collections::HashMap, sync::Arc};

use parking_lot::Mutex;
use tokio::sync::mpsc;

use crate::{
    expire::{self, Expiration},
    forward::{self, Forward},
    slot::Slot,
};

#[derive(Clone)]
pub struct BgTask {
    // 过期task
    pub expire_sender: mpsc::Sender<expire::Entry>,
    // 转发task
    pub forward_sender: mpsc::Sender<forward::Message>,
}
pub struct Db {
    bg_task: BgTask,
    slots: HashMap<u16, Slot>,
}

impl Db {
    pub fn new() -> Arc<Self> {
        let expiration = Expiration::new();
        let forward = Forward::new();
        let bg_task = BgTask {
            expire_sender: expiration.tx.clone(),
            forward_sender: forward.tx.clone(),
        };
        let mut slots = HashMap::new();
        for i in 0..1024 {
            slots
                .entry(i)
                .or_insert_with(|| Slot::new(i, bg_task.clone()));
        }
        let db = Arc::new(Self { bg_task, slots });
        expiration.listen(Arc::clone(&db));
        forward.listen();
        db
    }
}
