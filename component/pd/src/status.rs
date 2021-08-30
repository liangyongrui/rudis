use std::{net::SocketAddr, time::Duration};

use common::{
    now_timestamp_ms,
    pd_message::{ServerInit, ServerRole, ServerStatus},
};
use once_cell::sync::Lazy;
use parking_lot::RwLock;

static STATUS: Lazy<RwLock<Status>> = Lazy::new(|| RwLock::new(Status::default()));

#[derive(Default)]
struct Status {
    // 最后一个分配的group_id
    latest_group_id: usize,
    groups: Vec<Group>,
}

/// 分组信息，暂时只有这些
///
/// 以后会增加slot信息之类
struct Group {
    /// group id
    id: usize,
    /// 最后一个分配的server_id
    latest_server_id: usize,
    /// group 下的server列表，节点不会很多，操作的时候直接遍历就很快
    servers: Vec<Server>,
    /// leader id SocketAddr
    leader: Option<(usize, SocketAddr)>,
}

struct Server {
    id: usize,
    /// 最近一次收到心跳的毫秒时间戳
    latest_heartbeat: u64,
    /// 角色
    role: ServerRole,
    /// serverNode 的请求接收地址
    socket_addr: SocketAddr,
    /// 加入时间
    join_time: u64,
}

pub fn create_group() {
    let status = &mut *STATUS.write();
    status.latest_group_id += 1;
    status.groups.push(Group::new(status.latest_group_id))
}

/// 初始化服务器
pub fn server_init(data: ServerInit) -> common::Result<ServerStatus> {
    let status = &mut *STATUS.write();
    if let Some(g) = status.groups.iter_mut().find(|t| t.id == data.group_id) {
        g.latest_server_id += 1;
        let is_follower = g.servers.iter().any(|t| t.role == ServerRole::Leader);
        let role = if is_follower {
            ServerRole::Follower
        } else {
            ServerRole::Leader
        };
        let now = now_timestamp_ms();
        let server = Server {
            id: g.latest_server_id,
            latest_heartbeat: now,
            role,
            socket_addr: data.socket_addr,
            join_time: now,
        };
        if let ServerRole::Leader = role {
            g.leader = Some((server.id, server.socket_addr));
        }
        let res = ServerStatus {
            server_id: server.id,
            group_id: g.id,
            role,
            current_leader: g.leader,
        };
        g.servers.push(server);
        Ok(res)
    } else {
        Err("group not exists.".into())
    }
}
impl Status {
    /// 获取节点的最新状态
    pub fn _server_status(&self, group_id: usize, server_id: usize) -> Option<ServerStatus> {
        if let Some(g) = self.groups.iter().find(|t| t.id == group_id) {
            if let Some(s) = g.servers.iter().find(|t| t.id == server_id) {
                return Some(ServerStatus {
                    server_id: s.id,
                    group_id: g.id,
                    role: s.role,
                    current_leader: g.leader,
                });
            }
        }
        None
    }

    /// 获取节点的最新状态
    pub fn heartbeat_server(&mut self, group_id: usize, server_id: usize) -> Option<ServerStatus> {
        if let Some(g) = self.groups.iter_mut().find(|t| t.id == group_id) {
            if let Some(s) = g.servers.iter_mut().find(|t| t.id == server_id) {
                s.latest_heartbeat = now_timestamp_ms();
                return Some(ServerStatus {
                    server_id: s.id,
                    group_id: g.id,
                    role: s.role,
                    current_leader: g.leader,
                });
            }
        }
        None
    }
}

/// SEVER_HEARTBEAT
pub fn server_heartbeat(data: ServerStatus) -> Option<ServerStatus> {
    let status = &mut *STATUS.write();
    let res = status.heartbeat_server(data.group_id, data.server_id)?;
    if data != res {
        Some(res)
    } else {
        None
    }
}

/// leader 存活检查
pub async fn server_survival_check() {
    let mut interval = tokio::time::interval(Duration::from_secs(1));
    let ts = 3_000;
    loop {
        interval.tick().await;
        let now = now_timestamp_ms();
        let mut status = STATUS.write();

        for g in &mut status.groups {
            let mut update_leader = true;
            if let Some(leader) = g.servers.iter_mut().find(|s| s.role == ServerRole::Leader) {
                if now > ts + leader.latest_heartbeat {
                    leader.role = ServerRole::Follower;
                } else {
                    update_leader = false;
                }
            }
            if update_leader {
                let mut leader: Option<&mut Server> = None;
                for s in &mut g.servers {
                    // alive follower
                    if now <= ts + s.latest_heartbeat && s.role == ServerRole::Follower {
                        match leader {
                            Some(l) if s.join_time < l.join_time => leader = Some(s),
                            None => leader = Some(s),
                            _ => (),
                        }
                    }
                }
                if let Some(leader) = leader {
                    leader.role = ServerRole::Leader;
                    g.leader = Some((leader.id, leader.socket_addr));
                }
            }
        }
    }
}

impl Group {
    fn new(id: usize) -> Self {
        Self {
            id,
            latest_server_id: 0,
            servers: vec![],
            leader: None,
        }
    }
}
