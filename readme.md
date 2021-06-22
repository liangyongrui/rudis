# RCC

rust cloud cache

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

兼容 redis 6.2 命令

1. [x] set
1. [x] get
1. [x] psetex
1. [x] setex

## 不支持的命令

1. deprecated
   - GETSET
