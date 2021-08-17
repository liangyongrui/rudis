# Placement Driver

管理全局配置

n 个节点的主从，组成一个 group

每个节点启动的时候 告诉 pd，自己要加入哪个 group (指定 group_id)

pd 来负责管理 group

## 以后可能会有的 group 配置项

1. 支持的slot
1. 持久化配置
1. 自动拉起配置
1. 强一致性主从
