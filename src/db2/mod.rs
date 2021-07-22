use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
    sync::Arc,
};

use tokio::sync::mpsc;

use crate::{
    expire::{self, Expiration},
    forward::{self, Forward},
    slot::{cmd, data_type::SimpleType, dict, Slot},
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

const SIZE: u16 = 1024;

impl Db {
    pub fn new() -> Arc<Self> {
        let expiration = Expiration::new();
        let forward = Forward::new();
        let bg_task = BgTask {
            expire_sender: expiration.tx.clone(),
            forward_sender: forward.tx.clone(),
        };
        let mut slots = HashMap::new();
        for i in 0..SIZE {
            slots
                .entry(i)
                .or_insert_with(|| Slot::new(i, bg_task.clone()));
        }
        let db = Arc::new(Self { bg_task, slots });
        expiration.listen(Arc::clone(&db));
        forward.listen();
        db
    }
    fn get_slot(&self, key: &SimpleType) -> &Slot {
        // todo 更完善的分片策略
        let mut s = DefaultHasher::new();
        key.hash(&mut s);
        let i = s.finish() % SIZE as u64;
        // todo cluster move
        self.slots.get(&(i as u16)).unwrap()
    }
}

/// cmd
impl Db {
    pub fn get(&self, cmd: cmd::get::Get) -> crate::Result<SimpleType> {
        self.get_slot(cmd.key).get(cmd)
    }
    pub async fn set(&self, cmd: cmd::set::Set) -> crate::Result<SimpleType> {
        self.get_slot(&cmd.key).set(cmd).await
    }
    pub async fn del(&self, cmd: cmd::del::Del) -> crate::Result<Option<dict::Value>> {
        self.get_slot(&cmd.key).del(cmd).await
    }
    pub async fn expire(&self, cmd: cmd::expire::Expire) -> crate::Result<bool> {
        self.get_slot(&cmd.key).expire(cmd).await
    }
}
