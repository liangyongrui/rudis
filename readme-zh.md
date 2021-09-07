# Rudis

**ru**sty **di**ctionary **s**erver

![test](https://github.com/liangyongrui/rudis/workflows/CI/badge.svg) ![coverage](https://codecov.io/gh/liangyongrui/rudis/branch/main/graph/badge.svg)

[English](./readme.md)

## Introduction

Rudis 是一款用 Rust 编写，多线程处理，兼容 redis 的数据库。它有着比 Redis 更优秀的性能，解决了一些 Redis 的常见问题。

## 目标 (WIP)

1. 兼容 redis client

1. 比 redis 的效率更高

   - 用 rust 实现，可以写出更安全高效的多线程数据引擎
   - 更高的 io 效率
   - 对大 key 优化
   - 对热 key 优化

1. 比 redis 的坑更少

   - 使用 rust 减少各种内存安全上的 bug
   - 过期数据及时清理

1. 比 redis 更好运维

   - 更容易管理的集群
   - 随意的横向扩容
   - 自动主从切换
   - 自动调度热 slot

## known issues, warnings

- **免责申明** 请暂时不要用于生产环境。
- 如果你很在意稳定性，推荐用[Redis](redis.io). Rudis is beta.
- PD 模块没有测试过，暂时不是高可用的，可能会出现单点故障问题
- 暂时只支持 linux 和 macos

## Quick start

1. 准备最新版的 [rust 工具链](https://rustup.rs/)
1. clone 代码
   - `git clone git@github.com:liangyongrui/rudis.git`
1. 依次执行
   - `cd rudis`
   - `cargo build --release`
   - `./target/release/server` (指定启动地址：`RUDIS_server_addr=0.0.0.0:6379 ./target/release/server`)

## Current roadmap (Version 0.1)

1. [ ] 通过 Redis TCL test
1. [ ] 完成高可用 pd

## Benchmarks

Rudis 的性能比 Redis 6.2.5 更好。
[查看详情 ](./docs/benchmark-zh.md)

## Supported Redis commands

[Supported redis commands](./docs/supported_redis_cmds.md)

## Contributing

Contributions are always welcomed! Please refer to [CONTRIBUTING](./CONTRIBUTING.md) for detailed guidelines.

## License

This project is licensed under the [MIT license](./LICENSE).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in rudis by you, shall be licensed as MIT, without any additional terms or conditions.
