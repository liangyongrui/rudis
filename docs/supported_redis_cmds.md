# Supported redis commands

All the implemented commands are compatible with redis 7.0 version.

| type | Command   | Support | Time Complexity |
| ---- | --------- | ------- | --------------- |
| base | del       | yes     | Same as redis   |
| base | set       | yes     | Same as redis   |
| base | get       | yes     | Same as redis   |
| base | psetex    | yes     | Same as redis   |
| base | setex     | yes     | Same as redis   |
| base | exists    | yes     | Same as redis.  |
| base | pexpireat | yes     | Same as redis   |
| base | expireat  | yes     | Same as redis   |
| base | expire    | yes     | Same as redis   |
| base | pexpire   | yes     | Same as redis   |
| base | incr      | yes     | Same as redis   |
| base | incrby    | yes     | Same as redis   |
| base | decr      | yes     | Same as redis   |
| base | decrby    | yes     | Same as redis   |
| base | ttl       | yes     | Same as redis   |
| base | pttl      | yes     | Same as redis   |

| type | Command | Support | Time Complexity        |
| ---- | ------- | ------- | ---------------------- |
| list | lpush   | yes     | Same as redis          |
| list | rpush   | yes     | Same as redis          |
| list | rpushx  | yes     | Same as redis          |
| list | lpushx  | yes     | Same as redis          |
| list | lrange  | yes     | O(STOP-START)          |
| list | lpop    | yes     | Same as redis returned |
| list | rpop    | yes     | Same as redis          |
| list | llen    | yes     | Same as redis          |

| type | Command | Support | Time Complexity |
| ---- | ------- | ------- | --------------- |
| hash | hget    | yes     | Same as redis   |
| hash | hmget   | yes     | Same as redis   |
| hash | hgetAll | yes     | Same as redis   |
| hash | hset    | yes     | Same as redis   |
| hash | hsetnx  | yes     | Same as redis   |
| hash | hdel    | yes     | Same as redis   |
| hash | hexists | yes     | Same as redis   |
| hash | hincrby | yes     | Same as redis   |

| type | Command    | Support | Time Complexity |
| ---- | ---------- | ------- | --------------- |
| set  | smismember | yes     | Same as redis   |
| set  | sismember  | yes     | Same as redis   |
| set  | sadd       | yes     | Same as redis   |
| set  | srem       | yes     | Same as redis   |
| set  | smembers   | yes     | Same as redis   |

| type | Command          | Support | Time Complexity                                                                                                                       |
| ---- | ---------------- | ------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| zset | zadd             | yes     | Same as redis                                                                                                                         |
| zset | zrange           | yes     | by_rank: O(N+M), other: O(log(N)+M) <br> with N being the number of elements in the sorted set and M the number of elements returned. |
| zset | zrevrank         | yes     | by_rank: O(N+M), other: O(log(N)+M) <br> with N being the number of elements in the sorted set and M the number of elements returned. |
| zset | zrank            | yes     |                                                                                                                                       |
| zset | zrem             | yes     |                                                                                                                                       |
| zset | zrevrange        | yes     |                                                                                                                                       |
| zset | zrangebyscore    | yes     |                                                                                                                                       |
| zset | zrevrangebyscore | yes     |                                                                                                                                       |
| zset | zrangebylex      | yes     |                                                                                                                                       |
| zset | zrevrangebylex   | yes     |                                                                                                                                       |
| zset | zremrangebyrank  | yes     |                                                                                                                                       |
| zset | zremrangebyscore | yes     |                                                                                                                                       |
