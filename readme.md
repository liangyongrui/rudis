# RCC

rust cloud cache

## 特点

1. 异步删除（不会再阻塞 get）
1. 并发

## todo

1. [x] 多 slot (shard)
   - [ ] 热 key 单独 slot（加锁转移）
1. [ ] 复杂数据结构(大 key), 持久化数据结构 mvcc
1. [ ] 单 key，多次更新聚合
1. [ ] 持久化
1. [ ] 主备
1. [ ] pipeline(停止服务器的时候，处理干净 pipeline)
1. [ ] 异步锁 (shutdown 改成原子的可能就好了)
1. [ ] 支持[resp3 协议](https://www.zeekling.cn/articles/2021/01/10/1610263628832.html)

## 支持的命令

所有已经实现的命令都是兼容 redis 6.2 的版本

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
1. [ ] incr
1. [ ] incrby
1. [ ] hgetAll
1. [ ] hset
1. [ ] rpush
1. [ ] sismember
1. [ ] sadd
1. [ ] hsetnx
1. [ ] hdel
1. [ ] smembers
1. [ ] lrange
