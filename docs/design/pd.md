# Placement Driver

管理全局配置

raft 保障高可用，可能使用的库

- <https://github.com/ritelabs/riteraft>
- <https://github.com/async-raft/async-raft>

## 设计原则

1. http 管理
   - 方便做界面管理
   - 所以对 pd 的请求上行请求尽可能的少
1. 尽可能的轻量，便于和 server node 混部
1. 核心逻辑单线程处理，减少复杂度

## 整体架构

1. 一个集群有多个 group
1. n 个节点组成一个 group
1. 每个节点启动的时候 告诉 pd，自己要加入哪个 group (指定 group_id)
1. 每个节点在 group 中多种角色
   - leader：该 group 的 leader
   - follower：该 group 的 follower (具体见 replication 的文档)

## 以后可能会有的 group 配置项

1. 支持的 slot
1. 持久化配置
1. 强一致性主从

## 能力

- [ ] server 节点通过 pd 注册启动
- [ ] 协调主从
- [ ] 协调 slot
- [ ] 管理 group 级别的配置
- [ ] 通知 proxy 主从、slot 的变更
- [ ] 通知 client 主从、slot、proxy 的变更
- [ ] 负载均衡 proxy，让 client 切换 proxy
- [ ] 负载均衡 slot，动态迁移高频 slot
- [ ] 提供分布式锁的能力?
- [ ] 管理多个集群？

## server 注册到 pd 的流程

1. server -> server: 初始化，保证 socket 可用
1. node -> pd: node 注册，携带的信息
   - group_id: 想要加入的组
   - role: 想要成为的角色
     - 目前只有一种 Candidate
   - node_id: 如果有的话 (连接意外断开重连的时候有)
   - socket listener 地址
1. pd -> node: pd 返回注册结果
   - success 是否成功，失败可能的原因
     - node_id 不存在
     - group_id 不存在
   - role: 角色信息, pd 自动分配你是 leader 还是 follower

## 心跳流程

1. pd -> node: pd 来轮询 node, ping
1. node -> pd: pong
1. pd -> pd: 一秒一次，三秒没有 pong，视为节点挂掉

## 当前实现的能力

1. [ ] 一个单点的 pd
1. [ ] 创建 group
1. [ ] 节点的心跳，动态切换主从
