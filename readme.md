# RCC

rust cloud cache

(希望有人能起个更好听的名字)

## 特点

1. [ ] 兼容所有的 redis client
   - 不需要 redis cluster 专用客户端
1. [x] 并发性
   - [x] 读写公平锁
   - [x] 单次请求只持有一次锁，并且时间粒度尽量小
   - [] slot 之间并发，空间粒度也尽量小
   - [ ] 大 key COW
1. [x] 过期异步删除
   - [x] 大 key 也是 O(1)的删除
1. [x] ha
1. [x] 自动化运维

## Benchmarks

[see detail](./docs/benchmark.md)

## 开源前

### 目标

不带集群功能的简单 redis

### deadline

2021-09-30

### todo list

1. [ ] 单点 pd，后期改为 raft
1. [ ] 主从
   - [ ] 可靠的主从复制
   - [ ] pd, 用来协调主从
1. [ ] fsync
   - [ ] tokio-uring
1. [ ] 持久化恢复
1. [ ] 错误信息
   - [ ] wrong number of arguments (given 3, expected 2)
   - [ ] error type
1. [ ] 各个 task 的优雅退出
1. [ ] 启动配置
1. [ ] u64 key map
1. [ ] make file
1. [ ] roadmap
1. [ ] 起个好名字
   - rudis
   - redis_iox
   - redox (喜欢这个，但是被别的项目用了。。)
1. [ ] 英文 readme
1. [ ] icon

## todo

1. [ ] HashTag
1. [ ] 持久化
   - aof + rdb 可能不是那么好
   - 在有主从复制的情况下，可能也没那么必要？
   - 尝试 diskstore
   - 修改不存在的值可能会多一次硬盘查询
     - 保存 key 加速?
     - 布隆过滤器加速?
1. [ ] pd
   - [raft](https://github.com/ritelabs/riteraft)
1. [ ] 代码覆盖率超过 90%
1. [ ] memtier_benchmark -n 10000 -c 200 -t 4 -R --hide-histogram
1. [ ] Redis TCL test
1. [ ] pd
   - slot, 查看监控、统计信息、同步代理等
   - 提供分布式锁的能力
1. [ ] 异步 drop (del 或者 被 set 覆盖, 都是异步 drop)
1. [ ] crc16
1. [ ] 连接权限管理
1. [x] aof
   - [x] 写
   - [ ] 读
1. [ ] cluster 模式
1. [ ] 代码设置默认配置
1. [ ] 内存不够时候的淘汰机制
1. [ ] 自定义插件
1. [ ] lua 脚本
1. [ ] 支持[resp3 协议](https://www.zeekling.cn/articles/2021/01/10/1610263628832.html)
1. [ ] 各种模块的测试
1. [ ] 分布式事务
1. [ ] db 和 slot 的模板代码 换成宏
1. [ ] 各种运行时监控
1. [ ] Keyspace Notification

## 待探索方向


## Supported redis commands

[Supported redis commands](./docs/supported_redis_cmds.md)
