# RCC

rust cloud cache

(希望有人能起个更好听的名字)

## 特点

1. rust 编写，更少的错误
1. 更好的性能
1. 更好的集群模式
1. 大 key set del 返回无阻塞

## todo list

1. [ ] 不需要插入的 key 不用 arc
1. [ ] servo_arc = "0.1.1"
1. [ ] 除了 key，别的都用 box<[u8]>
1. [ ] loop 向上抛的异常
1. [ ] 各个 task 的优雅退出
1. [ ] 规范每个模块的命名
1. [ ] 优化各种 error anyhow+thiserror
1. [ ] 检查代码中各种可能的 panic
1. [ ] 规范 tokio task 的使用
1. [ ] 再次检查主从复制的逻辑
1. [ ] 去掉中文注释, 尽可能的增加英文注释
1. [ ] roadmap
1. [ ] 起个好名字
   - rcc (**r**usty **c**loud **c**ache)
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
