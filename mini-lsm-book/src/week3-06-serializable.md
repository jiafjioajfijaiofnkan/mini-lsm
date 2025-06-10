<!--
  mini-lsm-book © 2022-2025 by Alex Chi Z is licensed under CC BY-NC-SA 4.0
-->

# （部分）可串行化快照隔离 ((A Partial) Serializable Snapshot Isolation)

现在，我们将在事务提交时添加一个冲突检测算法，以使引擎具有一定程度的可串行化能力。

要运行测试用例：

```
cargo x copy-test --week 3 --day 6
cargo x scheck
```

让我们来看一个可串行化的例子。假设引擎中有两个事务：

```
txn1: put("key1", get("key2"))
txn2: put("key2", get("key1"))
```

数据库的初始状态是 `key1=1, key2=2`。可串行化意味着执行的结果与按某种顺序串行执行事务的结果相同。如果我们先执行 txn1 然后执行 txn2，我们将得到 `key1=2, key2=2`。如果我们先执行 txn2 然后执行 txn1，我们将得到 `key1=1, key2=1`。

然而，在我们当前的实现中，如果这两个事务的执行发生重叠：

```
txn1: get key2 <- 2
txn2: get key1 <- 1
txn1: put key1=2, commit
txn2: put key2=1, commit
```

我们将得到 `key1=2, key2=1`。这无法通过这两个事务的任何串行执行顺序产生。这种现象称为写偏斜 (write skew)。

通过可串行化验证，我们可以确保对数据库的修改对应于串行执行顺序，因此，用户可以在需要可串行化执行的系统上运行一些关键工作负载。例如，如果用户在 Mini-LSM 上运行银行转账工作负载，他们会期望任何时间点的资金总额都是相同的。没有可串行化检查，我们无法保证此不变量。

一种可串行化验证技术是在系统中记录每个事务的读取集 (read set) 和写入集 (write set)。我们在提交事务之前进行验证（乐观并发控制）。如果事务的读取集与其读取时间戳之后提交的任何事务的写入集发生重叠，则验证失败，并中止该事务。

回到上面的例子，如果我们有两个事务 txn1 和 txn2，它们都在时间戳 = 1 时启动。

```
txn1: get key2 <- 2
txn2: get key1 <- 1
txn1: put key1=2, 提交时间戳 (commit ts) = 2
txn2: put key2=1, 开始可串行化验证
```

当我们验证 txn2 时，我们将检查在其自身预期提交时间戳之前且在其读取时间戳之后（在本例中为 1 < ts < 3）提交的所有事务。唯一满足此条件的事务是 txn1。txn1 的写入集是 `key1`，txn2 的读取集是 `key1`。由于它们重叠，我们应该中止 txn2。

## 任务 1：在 Get 和 Write Set 中跟踪读取集 (Track Read Set in Get and Write Set)

在此任务中，您需要修改：

```
src/mvcc/txn.rs
src/mvcc.rs
```

当调用 `get` 时，您应该将键添加到事务的读取集中。在我们的实现中，我们存储键的哈希值，以减少内存使用并加快探测读取集的速度，尽管当两个键具有相同的哈希值时，这可能会导致误报 (false positive)。您可以使用 `farmhash::hash32` 为键生成哈希值。请注意，即使 `get` 返回未找到键，该键仍应在读取集中进行跟踪。

在 `LsmMvccInner::new_txn` 中，如果 `serializable=true`，您应该为事务创建一个空的读取/写入集。

## 任务 2：在 Scan 中跟踪读取集 (Track Read Set in Scan)

在此任务中，您需要修改：

```
src/mvcc/txn.rs
```

在本课程中，我们仅保证 `get` 请求的完全可串行化。您仍然需要跟踪扫描的读取集，但在某些特定情况下，您可能仍会得到不可串行化的结果。

为了理解为什么这很困难，让我们看下面的例子。

```
txn1: put("key1", len(scan(..)))
txn2: put("key2", len(scan(..)))
```

如果数据库的初始状态为 `a=1,b=2`，我们应该得到 `a=1,b=2,key1=2,key2=3` 或 `a=1,b=2,key1=3,key2=2`。然而，如果事务执行如下：

```
txn1: len(scan(..)) = 2
txn2: len(scan(..)) = 2
txn1: put key1 = 2, commit, 读取集 = {a, b}, 写入集 = {key1}
txn2: put key2 = 2, commit, 读取集 = {a, b}, 写入集 = {key2}
```

这通过了我们的可串行化验证，并且不对应于任何串行执行顺序！因此，一个功能齐全的可串行化验证需要跟踪键范围，如果仅调用 `get`，则使用键哈希可以加速可串行化检查。有关如何正确实现可串行化检查，请参阅奖励任务。

## 任务 3：引擎接口和可串行化验证 (Engine Interface and Serializable Validation)

在此任务中，您需要修改：

```
src/mvcc/txn.rs
src/lsm_storage.rs
```

现在，我们可以继续在提交阶段实现验证。每次处理事务提交时，都应获取 `commit_lock`。这确保只有一个事务进入事务验证和提交阶段。

您需要检查提交时间戳在 `(read_ts, expected_commit_ts)`（两端均不包含）范围内的所有事务，并查看当前事务的读取集是否与任何满足此条件的事务的写入集重叠。如果我们能够提交事务，则提交一个写入批处理，并将此事务的写入集插入到 `self.inner.mvcc().committed_txns` 中，其中键是提交时间戳。

如果 `write_set` 为空，您可以跳过检查。只读事务总是可以提交的。

您还需要修改 `LsmStorageInner` 中的 `put`、`delete` 和 `write_batch` 接口。我们建议您定义一个辅助函数 `write_batch_inner` 来处理写入批处理。如果 `options.serializable = true`，`put`、`delete` 和面向用户的 `write_batch` 应该创建一个事务，而不是直接创建一个写入批处理。您的写入批处理辅助函数还应返回一个 `u64` 提交时间戳，以便 `Transaction::Commit` 可以正确地将已提交的事务数据存储到 MVCC 结构中。

## 任务 4：垃圾回收 (Garbage Collection)

在此任务中，您需要修改：

```
src/mvcc/txn.rs
```

当您提交事务时，还可以清理已提交的事务映射，以删除所有低于水印的事务，因为它们不会参与任何未来的可串行化验证。

## 测试您的理解 (Test Your Understanding)

* 如果您有构建关系数据库的经验，可以考虑以下问题：假设我们基于 Mini-LSM 构建一个数据库，其中我们将关系表中的每一行存储为一个键值对（键：主键，值：序列化的行），并启用可串行化验证，那么数据库系统是否直接获得了 ANSI 可串行化隔离级别的能力？为什么是或为什么不是？
* 我们在这里实现的实际上是保证可串行化的写快照隔离 (write snapshot-isolation)（参见 [A critique of snapshot isolation](https://dl.acm.org/doi/abs/10.1145/2168836.2168853)）。是否存在某些情况，执行是可串行化的，但会被写快照隔离验证拒绝？
* 有些数据库声称它们通过仅跟踪 get 和 scan 中访问的键（而不是键范围）来支持可串行化快照隔离。它们真的能防止由幻读 (phantom) 引起的写偏斜吗？（好吧……实际上，我说的是 [BadgerDB](https://dgraph.io/blog/post/badger-txn/)。）

我们不提供这些问题的参考答案，欢迎在 Discord 社区中讨论它们。

## 奖励任务 (Bonus Tasks)

* **只读事务 (Read-Only Transactions)。** 启用可串行化后，我们需要跟踪事务的读取集。
* **精度/谓词锁定 (Precision/Predicate Locking)。** 可以使用范围而不是单个键来维护读取集。当用户扫描整个键空间时，这将非常有用。这也将为扫描启用可串行化验证。

{{#include copyright.md}}
