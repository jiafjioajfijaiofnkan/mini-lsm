<!--
  mini-lsm-book © 2022-2025 by Alex Chi Z is licensed under CC BY-NC-SA 4.0
-->

# 事务和乐观并发控制 (Transaction and Optimistic Concurrency Control)

在本章中，您将实现 `Transaction` 的所有接口。您的实现将为事务内的修改维护一个私有工作区，并批量提交它们，以便事务内的所有修改在提交之前仅对事务本身可见。我们仅在提交时检查冲突（即可串行化冲突），这就是乐观并发控制。

要运行测试用例：

```
cargo x copy-test --week 3 --day 5
cargo x scheck
```

## 任务 1：本地工作区 + Put 和 Delete (Local Workspace + Put and Delete)

在此任务中，您需要修改：

```
src/mvcc/txn.rs
```

现在，您可以通过将相应的键/值插入到 `local_storage`（一个没有键时间戳的 skiplist 内存表）来实现 `put` 和 `delete`。请注意，对于删除操作，您仍然需要将其实现为插入一个空值，而不是从 skiplist 中删除一个值。

## 任务 2：Get 和 Scan

在此任务中，您需要修改：

```
src/mvcc/txn.rs
```

对于 `get`，您应该首先探测本地存储。如果找到一个值，则根据它是否是删除标记返回该值或 `None`。对于 `scan`，您需要像在第 1.1 章中为没有键时间戳的内存表实现迭代器一样，为 skiplist 实现一个 `TxnLocalIterator`。您需要在 `TxnIterator` 中存储一个 `TwoMergeIterator<TxnLocalIterator, FusedIterator<LsmIterator>>`。最后，鉴于 `TwoMergeIterator` 会保留子迭代器中的删除标记，您需要修改您的 `TxnIterator` 实现以正确处理删除操作。

## 任务 3：提交 (Commit)

在此任务中，您需要修改：

```
src/mvcc/txn.rs
```

我们假设一个事务只会在单个线程上使用。一旦您的事务进入提交阶段，您应该将 `self.committed` 设置为 true，以便用户不能在该事务上执行任何其他操作。如果事务已提交，您的 `put`、`delete`、`scan` 和 `get` 实现应该报错。

您的提交实现应该简单地从本地存储中收集所有键值对，并向存储引擎提交一个写入批处理。

## 任务 4：原子 WAL (Atomic WAL)

在此任务中，您需要修改：

```
src/wal.rs
src/mem_table.rs
src/lsm_storage.rs
```

请注意，`commit` 涉及生成一个写入批处理，目前，写入批处理不保证原子性。您需要更改 WAL 实现以在写入批处理的开头和结尾生成头部和尾部。

新的 WAL 编码如下：

```
|   头部 (HEADER)   |                          主体 (BODY)                                      |  尾部 (FOOTER)  |
|     u32    |   u16   | var | u64 |    u16    |  var  |           ...            |    u32   |
| batch_size | key_len | key | ts  | value_len | value | 更多键值对... | checksum |
```

`batch_size` 是 `BODY` 部分的大小。`checksum` 是 `BODY` 部分的校验和。

没有测试用例来验证您的实现。只要您通过所有现有的测试用例并实现上述 WAL 格式，一切都应该没有问题。

您应该实现 `Wal::put_batch` 和 `MemTable::put_batch`。原始的 `put` 函数应将单个键值对视为一个批处理。也就是说，此时，您的 `put` 函数应该调用 `put_batch`。

一个批处理应该在同一个内存表和同一个 WAL 中处理，即使它超过了内存表的大小限制。

## 测试您的理解 (Test Your Understanding)

* 到目前为止我们实现的所有功能，系统是否满足快照隔离 (snapshot isolation)？如果不满足，我们还需要做什么来支持快照隔离？（注意：快照隔离与我们将在下一章讨论的可串行化快照隔离 (serializable snapshot isolation) 不同）
* 如果用户想要批量导入数据（例如 1TB）怎么办？如果他们使用事务 API 来执行此操作，您会给他们一些建议吗？针对这种情况是否有优化的机会？
* 什么是乐观并发控制 (optimistic concurrency control)？如果在 Mini-LSM 中实现悲观并发控制 (pessimistic concurrency control)，系统会是什么样子？
* 如果您的系统崩溃并在磁盘上留下损坏的 WAL 会怎样？您如何处理这种情况？
* 当您提交事务时，是否有必要将所有内容批量放入内存表，或者您可以逐个键地放入？为什么？

## 奖励任务 (Bonus Tasks)

* **溢出到磁盘 (Spill to Disk)。** 如果事务的私有工作区变得过大，您可以将一些数据刷写到磁盘。

{{#include copyright.md}}
