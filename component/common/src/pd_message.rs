use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

/// 传输的时候指定的协议
pub mod cmd {

    /// server init <ServerInit>
    pub const SEVER_INIT: &str = "pd_sever_init";

    /// server heartbeat <ServerStatus>
    pub const SEVER_HEARTBEAT: &str = "pd_sever_heartbeat";

    /// latest server status in pd <ServerStatus>
    pub const LATEST_SERVER_STATUS: &str = "pd_latest_server_status";

    /// create new group
    pub const CRATE_GROUP: &str = "pd_crate_group";
}

/// server role
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
pub enum ServerRole {
    Leader,
    Follower,
}

impl Default for ServerRole {
    fn default() -> Self {
        ServerRole::Leader
    }
}

/// server 状态
///
/// 用于
/// - server 上报
/// - pd 回复初始化
/// - pd 发现server上报的和实际不一致回复
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct ServerStatus {
    pub server_id: usize,
    pub group_id: usize,
    pub role: ServerRole,
    /// leader server id, leader forward addr
    pub current_leader: Option<(usize, SocketAddr)>,
}

/// server 初始化请求
#[derive(Debug, Deserialize, Serialize)]
pub struct ServerInit {
    pub group_id: usize,
    pub server_addr: SocketAddr,
    pub forward_addr: SocketAddr,
}
