# RCC

rust cloud cache

## 特点

1. 完全并发
1. 过期异步删除

## 整体设计

- group: 一个 group 为一个主节点+多个从节点
  - 主 可以写和读
  - 从 只能读
  - 主从之间通过 raft 来调度
- 整个集群分成 x 个 slot, 每个 group 包含多个 slot

## todo

1. [ ] https://github.com/dalei2019/redis-study/blob/main/docs/redis-rdb-format.md
1. [x] 多 slot (shard)
1. [x] 复杂数据结构, 持久化数据结构 mvcc
1. [x] nom parse
1. [x] macros
1. [x] 接收字符串的地方 都改成 基础类型(string, blob, i64, f64)
1. [x] hds(rdb)
1. [ ] 实现主从
   - [ ] 全量同步
   - [ ] 部分重同步
   - [ ] 密码验证
   - [ ] replica 的 replica 自动挂到master上
1. [ ] cluster 模式
   - [ ] 支持主从读写分离
1. [ ] 集群 proxy(可能用 raft)
1. [ ] #[instrument] 用法
1. [ ] 多个建立连接同时请求报错
1. [x] 测试命令 demo
1. [ ] 测试各个命令
1. [ ] sorted set 模块测试
1. [ ] 高可用
1. [ ] 单 key，多次更新聚合
1. [ ] 持久化
1. [ ] 主备
1. [ ] 自定义插件
1. [ ] lua 脚本
1. [ ] 支持[resp3 协议](https://www.zeekling.cn/articles/2021/01/10/1610263628832.html)
1. [ ] 整体梳理错误处理
1. [ ] 完备的测试
1. [ ] 注释中加测试
1. [ ] 带 size 的持久化红黑树/btree/b+tree
1. [ ] 在 slot 上加一个 tokio 的大锁，内部无锁, 测试这种方案的效率
1. [ ] 各种动态配置参数
1. [ ] <https://jzwdsb.github.io/2019/01/CRDT/>
1. [ ] [优化rdb保存结构](https://github.com/dalei2019/redis-study/blob/main/docs/redis-rdb-format.md) 

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
   - 根据排名查询的时候，假设范围是 m 到 n 时间复杂度为 O(n)
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
