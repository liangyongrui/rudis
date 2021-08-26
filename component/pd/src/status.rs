// //! 全局状态

// use common::now_timestamp_ms;
// use once_cell::sync::Lazy;
// use parking_lot::Mutex;

// static STATUS: Lazy<Mutex<Status>> = Lazy::new(|| Mutex::new(Status::default()));

// #[derive(Default)]
// struct Status {
//     groups: Vec<Group>,
// }

// impl Status {
//     pub fn add_group(&mut self) {
//         self.groups.push(Group::new(self.groups.len()))
//     }
// }

// /// 分组信息，暂时只有这些
// ///
// /// 以后会增加slot信息之类
// struct Group {
//     /// group id 和 groups的下标保持一致
//     id: usize,
//     // 最后一个分配的server_id
//     lastet_server_id: usize,
//     /// group 下的server列表，节点不会很多，操作的时候直接遍历就很快
//     servers: Vec<Server>,
// }

// impl Group {
//     fn new(id: usize) -> Self {
//         Self {
//             id,
//             lastet_server_id:0,
//             servers: vec![],
//         }
//     }

//     fn add_candidate(&mut self, socket_addr: String) -> Role {
//         self.lastet_server_id += 1;
//         let server_id = (self.id << 10) + self.lastet_server_id;
//         let is_follower = self.servers.iter().any(|t| matches!(t.role, Role::Leader));
//         let role = if is_follower {Role::Follower} else {Role::Leader};
//         let server= Server {
//             id:server_id,
//             latest_heartbeat:now_timestamp_ms(),
//             role
//         }
//     }
// }

// struct Server {
//     id: usize,
//     /// 最近一次收到心跳的毫秒时间戳
//     latest_heartbeat: u64,
//     /// 角色
//     role: Role,
//     /// serverNode 的请求接收地址，形如 "0.0.0.0:1000"
//     socket_addr: String,
// }
// enum Role {
//     Leader,
//     Follower,
// }
