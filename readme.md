# RCC

rust cloud cache

## 特点

1. 异步删除（不会再阻塞 get）
1. 并发

## todo

1. [x] 多 slot (shard)
   - [ ] 热 key 单独 slot（加锁转移），动态计算持有锁的时间
1. [ ] 复杂数据结构(大 key), 持久化数据结构 mvcc
1. [ ] 单 key，多次更新聚合
1. [ ] 持久化
1. [ ] 主备
1. [ ] pipeline(停止服务器的时候，处理干净 pipeline)
1. [ ] 异步锁 (shutdown 改成原子的可能就好了)
1. [ ] 支持[resp3 协议](https://www.zeekling.cn/articles/2021/01/10/1610263628832.html)

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
1. [ ] ltrim
1. [ ] lset
1. [ ] lindex

hash

1. [ ] hgetAll
1. [ ] hset
1. [ ] hsetnx
1. [ ] hdel

set

1. [ ] sismember
1. [ ] sadd
1. [ ] srem
1. [ ] smembers

zset

1. [ ] zrange
1. [ ] zrevrank
1. [ ] zadd
1. [ ] zremrangebyrank
1. [ ] zrevrangebyscore
