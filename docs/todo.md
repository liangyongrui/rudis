# TODO

1. [ ] make file
   - [ ] 启动配置
1. [ ] 兼容 redis cluster
1. [ ] 持久化
1. [ ] 完善 pd
1. [ ] test
   1. [ ] 代码覆盖率超过 90%
   1. [ ] memtier_benchmark -n 10000 -c 200 -t 4 -R --hide-histogram
   1. [ ] Redis TCL test
1. [ ] 异步 drop (del 或者 被 set 覆盖, 都是异步 drop)
1. [ ] 连接权限管理
1. [ ] 内存不够时候的淘汰机制
1. [ ] 自定义插件
1. [ ] lua 脚本
1. [ ] 支持[resp3 协议](https://www.zeekling.cn/articles/2021/01/10/1610263628832.html)
1. [ ] 各种模块的测试
1. [ ] 支持多 key 命令，事务
1. [ ] db 和 slot 的模板代码 换成宏
1. [ ] 各种运行时监控
1. [ ] Keyspace Notification
1. [ ] [acl](https://redis.io/topics/acl)

## 一些需要探索的方向

(可能有用的优化, tradeoff 的优化, 没想清楚的优化)

1. 单 slot 操作聚合
   - 避免频繁加锁, 增加吞吐
   - key 操作聚合
1. [使用其他内存分配器](https://poly000.github.io/perf-book-zh/heap-allocations_zh.html#%E4%BD%BF%E7%94%A8%E5%85%B6%E4%BB%96%E5%88%86%E9%85%8D%E5%99%A8)
1. 根据 value 的大小动态调整数据结构
   - rpds
1. 更高效的并发模型
   - 比如持久化数据结构
1. 集群事务
1. 从节点直接持久化保存, 减少从节点的内存成本
1. 主从多对一
1. 更可靠的主从复制
   1. 强一致性主从复制
      - 可能会增加单次耗时
      - 如果并发量比较大的话，吞吐量应该影响不大
1. 多主(多写)
   - [crdt](https://josephg.com/blog/crdts-go-brrr/)
   - [可能可以考虑用这个](https://github.com/josephg/diamond-types)
1. key 优化
   - <https://github.com/BurntSushi/bstr>
   - 比如 arc<[u8]>, 可以精简一个 weak reference, 每个 key 节约一个 byte
   - <https://doc.rust-lang.org/nomicon/arc.html>
   - 小 key 可能不用 引用计数，直接 copy 就好, 每个 key 节约三个 byte, 如果是用 0 在结尾，还能再省点
1. [hashmap 优化](https://youtu.be/ncHmEUmJZf4?t=2861)
1. bit 类型 + [slab](https://docs.rs/slab/)
1. 热 slot 自动迁移
1. 直接用 rocketdb 存储
1. 不同的命令用不同的一致性策略，比如 xxx 命令用强一致性主从
