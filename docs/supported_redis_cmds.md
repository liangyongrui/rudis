# Supported redis commands

All the implemented commands are compatible with redis 7.0 version.

## base

1. del
1. set
1. get
1. psetex
1. setex
1. exists
1. pexpireat
1. expireat
1. expire
1. pexpire
1. incr
1. incrby
1. decr
1. decrby
1. ttl
1. pttl

## list

1. lpush
1. rpush
1. rpushx
1. lpushx
1. lrange : O(STOP-START)
1. lpop
1. rpop
1. llen

## hash

1. hget
1. hmget
1. hgetAll
1. hset
1. hsetnx
1. hdel
1. hexists
1. hincrby

## set

1. smismember
1. sismember
1. sadd
1. srem
1. smembers

## zset

1. zadd
1. zrange : by_rank: O(N+M), other: O(log(N)+M) with N being the number of elements in the sorted set and M the number of elements returned.
1. zrevrank : O(N) N is rank
1. zrank : O(N) N is rank
1. zrem
1. zrevrange : by_rank: O(N+M), other: O(log(N)+M) with N being the number of elements in the sorted set and M the number of elements returned.
1. zrangebyscore
1. zrevrangebyscore
1. zrangebylex
1. zrevrangebylex
1. zremrangebyrank : O(N+M) with N being the number of elements in the sorted set and M the number of elements returned.
1. zremrangebyscore

## server

1. flushall
1. info: return some fake data
1. DUMP: Data structure is not the same
1. RESTORE: not support [FREQ frequency]
1. debug: just response "ok"
1. config: just response "ok"
1. object: just support idletime
