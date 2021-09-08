# Frame To Cmd

让所有的命令解析都可以用宏实现

## 基础类型

- Key
- Box<[u8]>
- i64
- usize
- f64
- DataType

## 复合类型

- (基础类型, 基础类型)
- vec<基础类型>
  - 默认只有一个，但是需要可为空
- vec<(基础类型, 基础类型)>
  - 默认只有一个，但是需要可为空

## 目前的几个需求

1. 按顺序直接读取

1. N 选 一 参数
   - 支持默认值
   - 支持选项有 value, value 是`基础类型`或 `复合类型`
   - 特殊处理一下 bool 二选一
   - 比如：none nx xx，默认 none
   - 比如：[EX seconds|PX milliseconds|EXAT timestamp|PXAT milliseconds-timestamp|KEEPTTL]
   - 比如：[LIMIT offset count]

## 实现

应该只通过类型签名就可以完成
