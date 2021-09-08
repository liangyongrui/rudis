# Frame To Cmd

让所有的命令解析都可以用宏实现

## 基础类型

- Key
- &'a [u8]
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

1. N 选 1 参数
   - 支持默认值
   - 支持选项有 value, value 是`基础类型`或 `复合类型`
   - 特殊处理一下 bool 二选一
   - 比如：none nx xx，默认 none
   - 比如：[EX seconds|PX milliseconds|EXAT timestamp|PXAT milliseconds-timestamp|KEEPTTL]
   - 比如：[LIMIT offset count]

## 实现

应该只通过类型签名就可以完成

N 选 1 参数 用 enum 实现，

手动对 enum 实现 default, 如果没实现视为至少选一个

需要一个 enum 宏

成员上加个 Attribute，要不然没办法识别是枚举

enum 宏 提供的函数

1. 输出有哪些 tag (用于模式匹配)
1. 把 tag 和 parse 传进去得到 enum
