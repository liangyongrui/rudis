# Rudis

**ru**sty **di**ctionary **s**erver

![test](https://github.com/liangyongrui/rudis/workflows/CI/badge.svg) ![coverage](https://codecov.io/gh/liangyongrui/rudis/branch/main/graph/badge.svg)

[中文](./readme-zh.md)

## Introduction

Rudis is a database written in Rust, multi-threaded, and compatible with redis. It has better performance than Redis and solves some common problems of Redis.

## Goal (WIP)

1. Compatible with redis client

1. More efficient than redis

   - Implemented with rust, you can write a safer and more efficient multi-threaded data engine
   - Higher io efficiency
   - Optimized for big keys
   - Optimized for hot keys

1. Less problems than redis

   - Use rust to reduce various memory security bugs
   - Timely clean up expired data

1. Better operation and maintenance than redis

   - Easier to manage clusters
   - Random horizontal expansion
   - Automatic leader-follower switch
   - Automatic scheduling of hot slots

## known issues, warnings

- **Disclaimer** Please don't use rudis in production now.
- If reliability is your primary constraint, use [Redis](redis.io). Rudis is beta.
- The PD module has not been tested and is temporarily not highly available, so there may be a single point of failure problem
- Only supports linux and macos

## Quick start

1. Prepare the latest version of [rust toolchain](https://rustup.rs/)
1. Clone code
   - git clone git@github.com:liangyongrui/rudis.git
1. Execute in sequence
   - `cd rudis`
   - `cargo build --release`
   - `./target/release/server` (Specify address to start: `RUDIS_server_addr=0.0.0.0:6379 ./target/release/server`)

## Current roadmap (Version 0.1)

1. [ ] Pass Redis TCL test
1. [ ] Complete high-availability pd

## Benchmarks

The performance of Rudis is better than Redis 6.2.5.
[Check the details.](./docs/benchmark.md)

## Supported redis commands

[Supported redis commands](./docs/supported_redis_cmds.md)

## Contributing

Contributions are always welcomed! Please refer to [CONTRIBUTING](./CONTRIBUTING.md) for detailed guidelines.

## License

This project is licensed under the [MIT license](./LICENSE).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in rudis by you, shall be licensed as MIT, without any additional terms or conditions.
