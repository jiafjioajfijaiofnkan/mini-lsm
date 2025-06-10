<!--
  mini-lsm-book © 2022-2025 by Alex Chi Z is licensed under CC BY-NC-SA 4.0
-->

# 读取路径 (Read Path)

![本章概览](./lsm-tutorial/week1-05-overview.svg)

在本章中，您将：

* 将 SST (有序字符串表) 集成到 LSM 读取路径中。
* 使用 SST 实现 LSM 读取路径 `get`。
* 使用 SST 实现 LSM 读取路径 `scan`。

要将测试用例复制到入门代码并运行它们：

```
cargo x copy-test --week 1 --day 5
cargo x scheck
```

## 任务 1：双路合并迭代器 (Two Merge Iterator)

在此任务中，您需要修改：

```
src/iterators/two_merge_iterator.rs
```

您已经实现了一个合并迭代器，用于合并相同类型的迭代器（例如，内存表迭代器）。既然我们已经实现了 SST 格式，我们就同时拥有了磁盘上的 SST 结构和内存中的内存表。当我们从存储引擎扫描时，需要将来自内存表迭代器和 SST 迭代器的数据合并为一个。在这种情况下，我们需要一个 `TwoMergeIterator<X, Y>` 来合并两种不同类型的迭代器。

您可以在 `two_merge_iterator.rs` 中实现 `TwoMergeIterator`。由于这里我们只有两个迭代器，因此不需要维护二叉堆。相反，我们可以简单地使用一个标志来指示从哪个迭代器读取。与 `MergeIterator` 类似，如果两个迭代器中都找到相同的键，则第一个迭代器优先。

## 任务 2：读取路径 - Scan (Read Path - Scan)

在此任务中，您需要修改：

```
src/lsm_iterator.rs
src/lsm_storage.rs
```

实现 `TwoMergeIterator` 后，我们可以将 `LsmIteratorInner` 更改为以下类型：

```rust,no_run
type LsmIteratorInner =
    TwoMergeIterator<MergeIterator<MemTableIterator>, MergeIterator<SsTableIterator>>;
```

这样，我们 LSM 存储引擎的内部迭代器将是一个结合了来自内存表和 SST 数据的迭代器。

目前，我们的 SST 迭代器不支持扫描的结束边界。为了解决这个问题，您需要在 `LsmIterator` 内部实现此边界检查。这涉及到更新 `LsmIterator::new` 构造函数以接受 `end_bound` 参数：

```rust,no_run
pub(crate) fn new(iter: LsmIteratorInner, end_bound: Bound<Bytes>) -> Result<Self> {}
```

然后，您需要修改 `LsmIterator` 的迭代逻辑，以确保当内部迭代器的键达到或超过指定的 `end_bound` 时停止。

我们的测试用例将在 `l0_sstables` 中生成一些内存表和 SST，您需要在此任务中正确扫描所有这些数据。在下一章之前，您不需要刷写 SST。因此，您可以继续修改您的 `LsmStorageInner::scan` 接口，以创建所有内存表和 SST 的合并迭代器，从而完成存储引擎的读取路径。

因为 `SsTableIterator::create` 涉及 I/O 操作并且可能很慢，所以我们不希望在 `state` 临界区中执行此操作。因此，您应该首先获取 `state` 的读锁并克隆 LSM 状态快照的 `Arc`。然后，您应该释放锁。之后，您可以遍历所有 L0 SST 并为每个 SST 创建迭代器，然后创建一个合并迭代器来检索数据。

```rust,no_run
fn scan(&self) {
    let snapshot = {
        let guard = self.state.read();
        Arc::clone(&guard)
    };
    // 创建迭代器并进行寻址
}
```

在 LSM 存储状态中，我们仅在 `l0_sstables` 向量中存储 SST ID。您需要从 `sstables` 哈希映射中检索实际的 SST 对象。

## 任务 3：读取路径 - Get (Read Path - Get)

在此任务中，您需要修改：

```
src/lsm_storage.rs
```

对于 get 请求，它将被处理为在内存表中查找，然后在 SST 上扫描。在探测所有内存表之后，您可以创建所有 SST 的合并迭代器。您可以寻址到用户想要查找的键。寻址有两种可能性：键与用户探测的相同，或者键不同/不存在。只有当键存在且与探测的相同，并且值不为空时，才应将值返回给用户。您还应该像上一节中那样减少状态锁的临界区。另外请记住处理已删除的键。

## 测试您的理解 (Test Your Understanding)

* 考虑用户拥有一个迭代整个存储引擎的迭代器，并且存储引擎有 1TB 大，因此扫描所有数据大约需要 1 小时。如果用户这样做，会出现什么问题？（这是一个很好的问题，我们将在课程的不同阶段多次提问……）
* 一些 LSM 树存储引擎提供的另一个流行接口是 multi-get（或 vectored get）。用户可以传递他们想要检索的键的列表。该接口返回每个键的值。例如，`multi_get(vec!["a", "b", "c", "d"]) -> a=1,b=2,c=3,d=4`。显然，一个简单的实现是简单地为每个键执行一次 get 操作。您将如何实现 multi-get 接口，以及您可以进行哪些优化以使其更高效？（提示：get 过程中的某些操作对于所有键只需要执行一次，此外，您可以考虑改进磁盘 I/O 接口以更好地支持此 multi-get 接口）。

我们不提供这些问题的参考答案，欢迎在 Discord 社区中讨论它们。

## 奖励任务 (Bonus Tasks)

* **动态分派的成本 (The Cost of Dynamic Dispatch)。** 实现一个 `Box<dyn StorageIterator>` 版本的合并迭代器，并进行基准测试以查看性能差异。
* **并行寻址 (Parallel Seek)。** 创建合并迭代器需要加载所有底层 SST 的第一个块（当您创建 `SSTIterator` 时）。您可以并行化创建迭代器的过程。

{{#include copyright.md}}
