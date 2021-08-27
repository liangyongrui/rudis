use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

/// 传输的时候指定的协议
pub mod cmd {

    /// server init <ServerInit>
    pub const SEVER_INIT: &str = "PD_SEVER_INIT";

    /// server heartbeat <ServerStatus>
    pub const SEVER_HEARTBEAT: &str = "PD_SEVER_HEARTBEAT";

    /// lastet server status in pd <ServerStatus>
    pub const LASTEST_SERVER_STATUS: &str = "PD_LASTEST_SERVER_STATUS";

    /// server init fail <msg str>
    pub const SEVER_INIT_FAIL: &str = "PD_SEVER_INIT_FAIL";

    /// create new group
    pub const NEW_GROUP: &str = "PD_NEW_GROUP";
}

/// server role
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
pub enum ServerRole {
    Leader,
    Follower,
}

/// server 状态
///
/// 用于
/// - server 上报
/// - pd 回复初始化
/// - pd 发现server上报的和实际不一致回复
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ServerStatus {
    pub server_id: usize,
    pub group_id: usize,
    pub role: ServerRole,
    pub current_leader: Option<(usize, SocketAddr)>,
}

/// server 初始化请求
#[derive(Debug, Deserialize, Serialize)]
pub struct ServerInit {
    pub group_id: usize,
    pub socket_addr: SocketAddr,
}
