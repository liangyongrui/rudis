# benchmark

[中文](./benchmark-zh.md)

## Platform

Mac mini (2018)

- OS: macOS 11.5.1 (20G80)
- CPU: 3.2GHz 6-core Intel Core i7

The client and server are on the same machine.

## Test Cmd

```shell
redis-benchmark -t set,get,rpush,lpop,lrange,hset -n 1000000 -r 100000 -c 1000 --threads 4 --csv
```

## Result

It seems that the gap is not big, but there are a few points to note:

1. When testing `rudis`, the peak cpu utilization of `rudis` is about 330%, and `redis-benchmark` is almost 400%. That is to say, the stress test tool is full, and `rudis` is not full yet, but it is actually not pressed. The bottleneck of rudis.
1. When testing `redis`, the peak cpu utilization of `redis` is about 400%, and `redis-benchmark` is about 300%. That is, the stress test tool is not full, and `redis` is full.
1. But even so, the performance of `rudis` is still better than that of `redis`.
1. If possible, put `redis-benchmark` on another machine, `rudis` should be able to run better data.
1. If you have the conditions, use more threads, the performance of `rudis` will likely be better. But `redis` is limited by io, and the performance is basically that way.

### rudis

| test                               | rps       | avg_latency_ms | min_latency_ms | p50_latency_ms | p95_latency_ms | p99_latency_ms | max_latency_ms |
| ---------------------------------- | --------- | -------------- | -------------- | -------------- | -------------- | -------------- | -------------- |
| SET                                | 221582.09 | 3.239          | 0.472          | 3.151          | 5.007          | 5.895          | 8.455          |
| GET                                | 221533.02 | 3.229          | 0.352          | 3.175          | 4.735          | 5.855          | 6.839          |
| RPUSH                              | 221483.95 | 3.302          | 0.672          | 3.231          | 4.887          | 5.935          | 7.871          |
| LPOP                               | 209819.56 | 3.414          | 0.648          | 3.319          | 5.143          | 5.919          | 27.935         |
| HSET                               | 209951.70 | 3.420          | 1.032          | 3.375          | 4.807          | 5.911          | 8.503          |
| LPUSH (needed to benchmark LRANGE) | 209775.53 | 3.413          | 0.936          | 3.335          | 4.783          | 5.919          | 27.231         |
| LRANGE_100 (first 100 elements)    | 83948.96  | 6.092          | 2.680          | 5.703          | 7.567          | 8.279          | 161.279        |
| LRANGE_300 (first 300 elements)    | 38735.67  | 13.006         | 2.336          | 12.487         | 16.927         | 18.495         | 198.911        |
| LRANGE_500 (first 500 elements)    | 25843.13  | 19.248         | 2.464          | 19.167         | 25.071         | 27.135         | 101.183        |
| LRANGE_600 (first 600 elements)    | 21827.87  | 22.752         | 2.288          | 22.703         | 29.935         | 31.967         | 100.991        |

### redis 6.2.5

| test                               | rps       | avg_latency_ms | min_latency_ms | p50_latency_ms | p95_latency_ms | p99_latency_ms | max_latency_ms |
| ---------------------------------- | --------- | -------------- | -------------- | -------------- | -------------- | -------------- | -------------- |
| SET                                | 159744.41 | 5.363          | 1.880          | 5.455          | 7.367          | 8.879          | 26.751         |
| GET                                | 166417.05 | 5.012          | 1.808          | 5.239          | 6.511          | 7.247          | 29.199         |
| RPUSH                              | 166334.00 | 5.171          | 1.856          | 5.399          | 6.703          | 7.559          | 25.647         |
| LPOP                               | 159693.39 | 5.256          | 1.952          | 5.455          | 6.927          | 7.631          | 28.495         |
| HSET                               | 147907.11 | 5.612          | 2.120          | 5.823          | 7.279          | 8.031          | 35.967         |
| LPUSH (needed to benchmark LRANGE) | 153562.66 | 5.490          | 1.968          | 5.567          | 7.231          | 8.343          | 130.111        |
| LRANGE_100 (first 100 elements)    | 61282.02  | 13.901         | 2.632          | 13.911         | 17.103         | 18.527         | 139.007        |
| LRANGE_300 (first 300 elements)    | 29683.28  | 24.178         | 1.448          | 23.791         | 35.807         | 42.047         | 144.255        |
| LRANGE_500 (first 500 elements)    | 19749.96  | 35.496         | 1.424          | 35.167         | 53.695         | 62.303         | 129.215        |
| LRANGE_600 (first 600 elements)    | 16997.84  | 40.824         | 1.384          | 40.607         | 62.079         | 71.103         | 128.895        |
