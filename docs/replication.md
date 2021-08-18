# 主从复制

## 目标

1. 减少复制过程中的内存开销
   - 不缓冲 aof
   - 同一时间的脏页少
   - 边序列化边传输
1. 减少复制过程中的 io 开销
   - 不进行磁盘快照
   - 传递的数据格式小
1. 速度快
   - 不进行磁盘快照
   - 不缓冲 aof
   - 断开连接和切换 master 部分重同步
1. 避免主节点没有持久化，原地拉起的时候丢数据
   - 空数据状态（没有 node_id）不能成为 master

## 主要流程

@import "replication.puml"

## 问题

1. pd 挂了，怎么连接上新的 master
1. 心跳用不用持久化处理