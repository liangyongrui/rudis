# RCC

rust cloud cache

(希望有人能起个更好听的名字)

## 特点

1. rust 编写，更少的错误
1. 更好的性能
1. 更好的集群模式
1. [ ] 任意 key O(1) 删除

## todo list

1. [ ] 各个 task 的优雅退出
1. [ ] 规范每个模块的命名
1. [ ] thiserror
1. [ ] 检查代码中的 todo 和 各种可能的 panic
1. [ ] 再次检查主从复制的逻辑
1. [ ] roadmap
1. [ ] 起个好名字
   - rudis(**ru**sty **di**ctionary **s**erver)
   - redis_rs
   - redis_iox
   - redox (喜欢这个，但是被别的项目用了。。)
1. [ ] 英文 readme
1. [ ] icon
1. [ ] license

## Benchmarks

[see detail](./docs/benchmark.md)

## Supported redis commands

[Supported redis commands](./docs/supported_redis_cmds.md)

fork from <https://github.com/tokio-rs/mini-redis>
