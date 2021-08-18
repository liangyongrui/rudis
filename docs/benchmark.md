# benchmark

## 平台

在 Mac mini (2018)

- OS: macOS 11.5.1 (20G80)
- CPU: 3.2 GHz 六核 Intel Core i7

客户端和服务端在同一台机器

## Test Cmd

```shell
redis-benchmark -t set,get,rpush,lpop,lrange,hset,hmget,zadd -n 1000000 -r 100000 -c 1000 --threads 4
```

## Result
