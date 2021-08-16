# RCC

rust cloud cache

(希望有人能起个更好听的名字)

## 特点

1. 兼容所有的 redis client
   - 不需要 redis cluster 专用客户端
1. 并发性
   - 读写公平锁
   - 单次请求只持有一次锁，并且时间粒度尽量小
   - slot 之间并发，空间粒度也尽量小
   - 大 key COW
1. 过期异步删除
   - 大 key 也是 O(1)的删除
1. ha
1. 自动化运维
1. tracing
1. 更小的集群通信开销

## 初步性能测试

在 Mac mini (2018)

- OS: macOS 11.5.1 (20G80)
- CPU: 3.2 GHz 六核 Intel Core i7

客户端和服务端在同一台机器

- redis io-threads 4 线程（redis 的最好状态）
- rcc worker_threads 4 线程

1000 个连接同时 set 请求 20000 次（非 pipeline）

| server      | 耗时(s) 三次平均值 | 频率(Hz)        |
| ----------- | ------------------ | --------------- |
| rcc         | 95.559099695       | 209294.56288135 |
| redis 6.2.5 | 231.862762724      | 86257.921561157 |

rcc 差不多是 redis 的 2.4 倍

[测试代码](cmd_test/bin/simple_bench.rs)

## 开源前

### 目标

不带集群功能的简单 redis

### deadline

2021-09-30

### todo list

1. [ ] 通过详尽的测试
   - [ ] 代码覆盖率超过 90%
   - [ ] Redis TCL test
1. [ ] 启动配置
1. [ ] fsync
1. [ ] 更加模块化
1. [ ] 可靠的主从复制
1. [ ] 持久化恢复
1. [ ] 独立的管理服务
   - [ ] 用来协调主从
   - (开源后再做) slot, 查看监控、统计信息、同步代理等
1. [ ] 错误信息
   - [ ] wrong number of arguments (given 3, expected 2)
   - [ ] error type
1. [ ] 各个 task 的优雅退出
1. [ ] 性能测试
   - [ ] benchmark
   - Rust 性能手册
1. [ ] 起个好名字
   - rudis
   - redis_iox
   - redox (喜欢这个，但是被别的项目用了。。)
1. [ ] 英文 readme

## todo

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

## 待探索方向

1. 代理
   - 客户端代理
   - 远程代理
1. 热 key 请求并发聚合
1. 根据 value 的大小动态调整数据结构
   - rpds
1. 更高效的并发模型
   - 比如持久化数据结构
1. io_uring
1. 集群事务
1. 从节点直接持久化保存, 减少从节点的内存成本
1. 混合存储
1. 主从多对一
1. 更可靠的主从复制
   1. 强一致性主从复制
      - 可能会增加单次耗时
      - 如果并发量比较大的话，吞吐量应该影响不大
1. 多主(多写)
   - [crdt](https://josephg.com/blog/crdts-go-brrr/)
   - [可能可以考虑用这个](https://github.com/josephg/diamond-types)
1. key 优化
   - 比如 arc<[u8]>, 可以精简一个 weak reference, 每个 key 节约一个 byte
   - 小 key 可能不用 引用计数，直接 copy 就好, 每个 key 节约三个 byte, 如果是用 0 在结尾，还能再省点
1. [hashmap 优化](https://youtu.be/ncHmEUmJZf4?t=2861)
1. 热 slot 自动迁移

## Supported redis commands

[Supported redis commands](./docs/supported_redis_cmds.md)
