# RCC

rust cloud cache

## 特点

1. 兼容 redis client
1. 并发性
   - 读写公平锁
   - 单次请求只持有一次锁，并且时间粒度尽量小
   - slot 之间并发，空间粒度也尽量小
1. 过期异步删除
1. cluster
1. ha
1. 自动化运维

## 整体设计

- group: 一个 group 为一个主节点+多个从节点
  - 主 可以写和读
  - 从 只能读
  - 主从之间通过 raft 来调度
- 整个集群分成 x 个 slot, 每个 group 包含多个 slot

## todo

1. [x] 多 slot (shard)
1. [x] 复杂数据结构, 持久化数据结构 mvcc
1. [x] nom parse
1. [x] macros
1. [x] 接收字符串的地方 都改成 基础类型(string, blob, i64, f64)
1. [x] hds(rdb)
1. [ ] 所有操作都判断一下 key 是否过期，过期就不处理了（不用删除，等待异步删除即可）
1. [ ] aof
   - [x] 写
     - [ ] 混合持久化
     - [ ] appendfsync
   - [ ] 读
1. [ ] 实现主从
   - [ ] 全量同步
   - [ ] 部分重同步
   - [ ] 密码验证
   - [ ] replica 的 replica 自动挂到 master 上
1. [ ] cluster 模式
   - [ ] 支持主从读写分离
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
1. [ ] 支持[resp3 协议](https://www.zeekling.cn/articles/2021/01/10/1610263628832.html)
1. [ ] 整体梳理错误处理
1. [ ] 完备的测试
1. [ ] 注释中加测试
1. [ ] 双写
1. [ ] 当使用 Redis 命令对数据库进行读写时，服务器不仅会对键空间执行指定的读写操作，还会执行一些额外的维护操作
   - [ ] 在读取一个键之后（读操作和写操作都要对键进行读取），服务器会根据键是否存在来更新服务器的键空间命中（hit）次数或键空间不命中（miss）次数，这两个值可以在 INFO stats 命令的 keyspace_hits 属性和 keyspace_misses 属性中查看。
   - [ ] 在读取一个键之后，服务器会更新键的 LRU（最后一次使用）时间，这个值可以用于计算键的闲置时间，使用 OBJECTidletime 命令可以查看键 key 的闲置时间。
   - [ ] 如果服务器在读取一个键时发现该键已经过期，那么服务器会先删除这个过期键，然后才执行余下的其他操作
1. [ ] 持久化 deque

## 不一定要做

1. [ ] [优化 rdb 保存结构](https://github.com/dalei2019/redis-study/blob/main/docs/redis-rdb-format.md)
1. [ ] 在 slot 上加一个 tokio 的大锁，内部无锁, 测试这种方案的效率
1. [ ] <https://jzwdsb.github.io/2019/01/CRDT/>
1. [ ] 带 size 的持久化红黑树/btree/b+tree
1. [ ] lua 脚本
1. [ ] 集群 proxy

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
1. [ ] PERSIST
1. [ ] ttl

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
