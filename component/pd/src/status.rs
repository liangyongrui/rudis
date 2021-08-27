use std::time::Duration;

use common::now_timestamp_ms;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpStream, ToSocketAddrs},
    select,
};

static STATUS: Lazy<Mutex<Status>> = Lazy::new(|| Mutex::new(Status::default()));

#[derive(Default)]
pub struct Status {
    // 最后一个分配的group_id
    lastet_group_id: usize,
    groups: Vec<Group>,
}

pub async fn server_survival_check() {
    let mut interval = tokio::time::interval(Duration::from_secs(1));
    let ts = 3_000;
    loop {
        let now = now_timestamp_ms();
        for g in &mut STATUS.lock().groups {
            for s in &g.servers {
                if now > s.latest_heartbeat && now - s.latest_heartbeat > ts {
                    // todo dead
                    continue;
                }
            }
        }
        interval.tick().await;
    }
}

impl Status {
    pub fn add_group(&mut self) {
        self.lastet_group_id += 1;
        self.groups.push(Group::new(self.lastet_group_id))
    }
    pub fn add_server(
        &mut self,
        group_id: usize,
        socket_addr: String,
    ) -> common::Result<(usize, Role)> {
        match self.groups.iter_mut().find(|t| t.id == group_id) {
            Some(g) => Ok(g.add_candidate(socket_addr)),
            None => Err("group not exists".into()),
        }
    }
}

/// 分组信息，暂时只有这些
///
/// 以后会增加slot信息之类
struct Group {
    /// group id
    id: usize,
    // 最后一个分配的server_id
    lastet_server_id: usize,
    /// group 下的server列表，节点不会很多，操作的时候直接遍历就很快
    servers: Vec<Server>,
}

impl Group {
    fn new(id: usize) -> Self {
        Self {
            id,
            lastet_server_id: 0,
            servers: vec![],
        }
    }

    fn add_candidate(&mut self, socket_addr: String) -> (usize, Role) {
        self.lastet_server_id += 1;
        let server_id = (self.id << 10) + self.lastet_server_id;
        let is_follower = self.servers.iter().any(|t| matches!(t.role, Role::Leader));
        let role = if is_follower {
            Role::Follower
        } else {
            Role::Leader
        };
        let server = Server {
            id: server_id,
            latest_heartbeat: now_timestamp_ms(),
            role,
            socket_addr: socket_addr.clone(),
        };
        self.servers.push(server);
        tokio::spawn(init_heartbeat(socket_addr, self.id, server_id));
        (server_id, role)
    }
}
const PING_FRAME: &[u8] = b"PING\r\n";
async fn init_heartbeat(addr: impl ToSocketAddrs, group_id: usize, server_id: usize) {
    let (mut r, mut w) = TcpStream::connect(addr).await.unwrap().into_split();
    let buf = &mut [0u8; 10];
    let mut interval = tokio::time::interval(Duration::from_secs(1));
    loop {
        select! {
            res = r.read(buf) => {
                if res.unwrap() == 0 {
                    // EOF
                    break;
                }
                if let Some(g) = STATUS.lock().groups.iter_mut().find(|g| g.id == group_id) {
                    if let Some(s) = g.servers.iter_mut().find(|s| s.id == server_id) {
                        s.latest_heartbeat = now_timestamp_ms();
                        continue;
                    }
                }
                // server列表没有他，但是有他的心跳了。突然复活
                break;
            }
            _ = interval.tick() => {
                w.write_all(PING_FRAME).await.unwrap()
            }
        }
    }
}

struct Server {
    id: usize,
    /// 最近一次收到心跳的毫秒时间戳
    latest_heartbeat: u64,
    /// 角色
    role: Role,
    /// serverNode 的请求接收地址，形如 "0.0.0.0:1000"
    socket_addr: String,
}

#[derive(Clone, Copy)]
pub enum Role {
    Leader,
    Follower,
}
