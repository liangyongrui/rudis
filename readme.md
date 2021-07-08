# RCC

rust cloud cache

## 特点

1. 完全并发
1. 异步删除

## todo

1. [x] 多 slot (shard)
1. [x] 复杂数据结构, 持久化数据结构 mvcc
1. [x] nom parse
1. [ ] 整体梳理错误处理
1. [ ] 单 key，多次更新聚合
1. [ ] 持久化
1. [ ] 主备
1. [ ] pipeline(停止服务器的时候，处理干净 pipeline)
1. [ ] 自定义插件
1. [ ] 支持[resp3 协议](https://www.zeekling.cn/articles/2021/01/10/1610263628832.html)
1. [ ] 完备的测试
1. [ ] 注释中加测试
1. [ ] 带 size 的持久化红黑树/btree/b+tree

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
1. [ ] zremrangebyscore
