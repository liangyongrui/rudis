# RCC

rust cloud cache

(希望有人能起个更好听的名字)

## 特点

1. 兼容 redis client
1. 并发性
   - 读写公平锁
   - 单次请求只持有一次锁，并且时间粒度尽量小
   - slot 之间并发，空间粒度也尽量小
   - 大 key COW
1. 过期异步删除
   - 大 key 也是 O(1)的删除
1. cluster
1. ha
1. 自动化运维
1. tracing
1. 更小的集群通信开销

## 初步性能测试

在 mbp2019

- OS: macOS 11.4
- CPU: 2.6 GHz 六核 Intel Core i7

客户端和服务端在同一台机器, work thread 都 限制 4 线程 （redis 的最好状态）

1000 个连接同时 set 请求 10000 次（非 pipeline）

| server      | 耗时(s) 三次平均值 | 频率(Hz)      | 内存占用(GB) |
| ----------- | ------------------ | ------------- | ------------ |
| rcc         | 50.3837099175      | 198476.849291 | 3.91         |
| redis 6.2.5 | 252.750146012      | 39564.764483  | 3.14         |

rcc 差不多是 redis 的 5 倍

[测试代码](cmd_test/bin/simple_bench.rs)

## To be optimized

### Performance

1. [ ] 根据 value 的大小 和 读写规律 来使用 可持久化数据结构
   - [ ] 持久化 deque
   - [ ] 带 rank 的平衡树

### Code

1. [ ] db 和 slot 的模板代码 换成宏
1. [ ] cmd parse 了两次，有点冗余
1. [ ] 错误信息
1. [ ] 各个 task 的优雅退出

## bug

1. [ ] 多个建立连接同时请求报错 (cmd_test/tests/connect.rs)
   - 好像是 tokio test 的问题
1. [ ] 修复一下未处理的 Err 和 unwrap

## 内存占用优化

1. [ ] hashmap 内存扩容机制，导致内存有些浪费
   - 修改一下 hashmap, 把 node box 一下，但是多一个指针的开销, 如果小 key 比较多大概能节约 30%的内存, 但是实际情况可能没有那么多的小 key
   - 把过期时间从 dict 提出来，可能可以节约 10%（但也多出了一些 key 的开销）。实际使用中，大部分的 key 都是有过期时间的，不一定能减少内存使用，说不定还会增加
1. [ ] 有一些数据指针过大
   - 比如 arc<[u8]>, 可以精简一个 weak reference, 每个 key 节约一个 byte

## todo

1. [ ] 可选的 \[u8] 压缩
1. [ ] crc16
1. [ ] 连接权限管理
1. [x] aof
   - [x] 写
   - [ ] 读
1. [ ] cluster 模式
1. [ ] 集群管理服务，用来协调 主从、slot, 查看监控、统计信息等
1. [ ] 代码设置默认配置
1. [ ] 内存不够时候的淘汰机制
1. [ ] 自定义插件
1. [ ] lua 脚本
1. [ ] 支持[resp3 协议](https://www.zeekling.cn/articles/2021/01/10/1610263628832.html)
1. [ ] 各种模块的测试
1. [ ] 稳定的 hash 数
1. [ ] 多主
   - [crdt](https://josephg.com/blog/crdts-go-brrr/)
   - [可能可以考虑用这个](https://github.com/josephg/diamond-types)
1. [ ] 集群 proxy
   - 同 key 聚合 (可能要设计一个 set_queue)
1. [ ] 分布式事务

## 待探索方向

1. io_uring
1. 集群事务
1. 从节点直接持久化保存, 减少从节点的内存成本
1. 混合存储
1. 主从多对一

## 支持的命令

所有已经实现的命令都是兼容 redis 6.2 的版本

base

1. [x] set
1. [x] get
1. [x] psetex
1. [x] setex
1. [x] del
1. [x] exists
1. [x] pexpireat
1. [x] expireat
1. [x] expire
1. [x] pexpire
1. [x] incr
1. [x] incrby
1. [x] decr
1. [x] decrby

list

1. [x] lpush
1. [x] rpush
1. [x] rpushx
1. [x] lpushx
1. [x] lrange
1. [x] lpop
1. [x] rpop
1. [x] llen

hash

1. [x] hget
1. [x] hmget
1. [x] hgetAll
1. [x] hset
1. [x] hsetnx
1. [x] hdel
1. [x] hexists
1. [x] hincrby

set

1. [x] smismember
1. [x] sismember
1. [x] sadd
1. [x] srem
1. [x] smembers

zset

1. [x] zadd
1. [x] zrange
   - 根据排名查询的时候，假设范围是 m 到 n，时间复杂度为 O(n)
1. [x] zrevrank
   - O(n)
1. [x] zrank
   - O(n)
1. [x] zrem
1. [x] zrevrange
1. [x] zrangebyscore
1. [x] zrevrangebyscore
1. [x] zrangebylex
1. [x] zrevrangebylex
1. [x] zremrangebyrank
1. [x] zremrangebyscore
