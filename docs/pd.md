# Placement Driver

管理全局配置

pd 模块的命令处理逻辑是单线程，消费一个 mpsc

提供分布式锁的能力

数据量不大，减少复杂度

使用 raft

- <https://github.com/ritelabs/riteraft>
- <https://github.com/async-raft/async-raft>

## 管理主从

n 个节点的主从，组成一个 group

每个节点启动的时候 告诉 pd，自己要加入哪个 group (指定 group_id)

pd 来负责管理 group

## 以后可能会有的 group 配置项

1. 支持的 slot
1. 持久化配置
1. 强一致性主从

## 讨论几种可能会出现的分布式问题
