<!--
  mini-lsm-book © 2022-2025 by Alex Chi Z is licensed under CC BY-NC-SA 4.0
-->

# 水印和垃圾回收 (Watermark and Garbage Collection)

在本章中，您将实现必要的结构来跟踪用户正在使用的最低读取时间戳，并在执行压缩时从 SST 中收集未使用的版本。

要运行测试用例：

```
cargo x copy-test --week 3 --day 4
cargo x scheck
```

## 任务 1：实现水印 (Implement Watermark)

在此任务中，您需要修改：

```
src/mvcc/watermark.rs
```

水印 (Watermark) 是用于跟踪系统中最低 `read_ts` (读取时间戳) 的结构。当创建新事务时，它应调用 `add_reader` 以添加其读取时间戳进行跟踪。当事务中止或提交时，它应从水印中移除自身。当调用 `watermark()` 时，水印结构返回系统中的最低 `read_ts`。如果没有正在进行的事务，则简单地返回 `None`。

您可以使用 `BTreeMap` 来实现水印。它为每个 `read_ts` 维护一个计数器，记录有多少快照正在使用此读取时间戳。B树映射中不应包含读取器数量为 0 的条目。

## 任务 2：在事务中维护水印 (Maintain Watermark in Transactions)

在此任务中，您需要修改：

```
src/mvcc/txn.rs
src/mvcc.rs
```

您需要在事务开始时将 `read_ts` 添加到水印中，并在为事务调用 `drop` 时将其移除。

## 任务 3：压缩中的垃圾回收 (Garbage Collection in Compaction)

在此任务中，您需要修改：

```
src/compact.rs
```

既然我们有了系统的水印，就可以在压缩过程中清理未使用的版本。

* 如果键的某个版本高于水印，则保留它。
* 对于所有低于或等于水印的键版本，保留最新版本。

例如，如果我们有 watermark=3 和以下数据：

```
a@4=del <- 高于水印 (above watermark)
a@3=3   <- 低于或等于水印的最新版本 (latest version below or equal to watermark)
a@2=2   <- 可以移除，没有人会读取它 (can be removed, no one will read it)
a@1=1   <- 可以移除，没有人会读取它 (can be removed, no one will read it)
b@1=1   <- 低于或等于水印的最新版本 (latest version below or equal to watermark)
c@4=4   <- 高于水印 (above watermark)
d@3=del <- 如果压缩到最底层，则可以移除 (can be removed if compacting to bottom-most level)
d@2=2   <- 可以移除 (can be removed)
```

如果我们对这些键进行压缩，我们将得到：

```
a@4=del
a@3=3
b@1=1
c@4=4
d@3=del (如果压缩到最底层，则可以移除)
```

假设这些是引擎中的所有键。如果在 ts=3 时进行扫描，压缩前后我们都会得到 `a=3,b=1,c=4`。如果在 ts=4 时进行扫描，压缩前后我们都会得到 `b=1,c=4`。压缩*不会*也*不应该*影响读取时间戳 >= 水印的事务。

## 测试您的理解 (Test Your Understanding)

* 在我们的实现中，我们通过 `Transaction` 的生命周期自行管理水印（所谓的非托管模式, un-managed mode）。如果用户打算自行管理键时间戳和水印（例如，当他们拥有自己的时间戳生成器时），您需要在 write_batch/get/scan API 中执行哪些操作来验证他们的请求？我们之前是否有任何架构假设在这种情况下可能难以维持？
* 为什么我们需要在事务迭代器中存储一个 `Arc` of `Transaction`？
* 从 SST 文件中完全移除一个键的条件是什么？
* 目前，我们仅在压缩到最底层时才移除键。是否有其他更早的时间可以移除键？（提示：您知道所有级别中每个 SST 的起始/结束键。）
* 考虑用户创建一个长时间运行的事务，导致我们无法进行任何垃圾回收的情况。用户不断更新单个键。最终，单个 SST 文件中可能存在一个键的数千个版本。这将如何影响性能，您将如何处理？

## 奖励任务 (Bonus Tasks)

* **O(1) 水印 (O(1) Watermark)。** 您可以使用哈希映射或循环队列来实现均摊 O(1) 的水印结构。

{{#include copyright.md}}
