# Placement Driver

管理全局配置

raft 保障高可用，可能使用的库

- <https://github.com/ritelabs/riteraft>
- <https://github.com/async-raft/async-raft>

## 设计原则

1. redis 协议
1. 心跳传递自己的配置, 一秒一次
   - 每个节点维护自己需要维护的状态，然后心跳带上
   - pd 如果发现状态和当前的不同，则返回最新的状态
1. [ ] 更新的时候主动推送

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

- [x] server 节点通过 pd 注册启动
- [x] 协调主从
- [ ] 协调 slot
- [ ] 管理 group 级别的配置
- [ ] 通知 proxy 主从、slot 的变更
- [ ] 通知 client 主从、slot、proxy 的变更
- [ ] 负载均衡 proxy，让 client 切换 proxy
- [ ] 负载均衡 slot，动态迁移高频 slot
- [ ] 提供分布式锁的能力?
- [ ] 管理多个集群？
