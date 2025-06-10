<!--
  mini-lsm-book © 2022-2025 by Alex Chi Z is licensed under CC BY-NC-SA 4.0
-->

# 写入路径 (Write Path)

![本章概览](./lsm-tutorial/week1-05-overview.svg)

在本章中，您将：

* 实现带有 L0 刷写的 LSM 写入路径。
* 实现正确更新 LSM 状态的逻辑。


要将测试用例复制到入门代码并运行它们：

```
cargo x copy-test --week 1 --day 6
cargo x scheck
```

## 任务 1：将 Memtable 刷写到 SST (Flush Memtable to SST)

至此，我们已经准备好了所有内存结构和磁盘文件，存储引擎能够读取和合并来自所有这些结构的数据。现在，我们将实现将数据从内存移动到磁盘的逻辑（称为刷写, flush），并完成 Mini-LSM 第 1 周的课程。

在此任务中，您需要修改：

```
src/lsm_storage.rs
src/mem_table.rs
```

您需要修改 `LSMStorageInner::force_flush_next_imm_memtable` 和 `MemTable::flush`。在 `LSMStorageInner::open` 中，如果 LSM 数据库目录不存在，您需要创建它。要将内存表刷写到磁盘，我们需要做三件事：

* 选择一个要刷写的内存表。
* 创建一个与内存表对应的 SST 文件。
* 从不可变内存表列表中移除该内存表，并将 SST 文件添加到 L0 SST 中。

我们目前还没有解释什么是 L0（0 级）SST。通常，它们是直接由内存表刷写产生的 SST 文件集合。在本课程的第 1 周，磁盘上只会有 L0 SST。我们将在第 2 周深入探讨如何使用磁盘上的分层或分级结构有效地组织它们。

请注意，创建 SST 文件是一项计算密集型且成本高昂的操作。同样，我们不希望长时间持有 `state` 读/写锁，因为它可能会阻塞其他操作并在 LSM 操作中产生巨大的延迟峰值。此外，我们使用 `state_lock` 互斥锁来序列化 LSM 树中的状态修改操作。在此任务中，您需要仔细考虑如何使用这些锁来使 LSM 状态修改无竞争条件，同时最小化临界区。

我们没有并发测试用例，您需要自行仔细考虑您的实现。另外，请记住，不可变内存表列表中的最后一个内存表是最早的，也是您应该刷写的那个。

<details>

<summary>剧透：刷写 L0 伪代码</summary>

```rust,no_run
fn flush_l0(&self) {
    let _state_lock = self.state_lock.lock();

    let memtable_to_flush;
    let snapshot = {
        let guard = self.state.read();
        memtable_to_flush = guard.imm_memtables.last();
    };

    let sst = memtable_to_flush.flush()?;

    {
        let guard = self.state.write();
        guard.imm_memtables.pop();
        guard.l0_sstables.insert(0, sst);
    };

}
```

</details>

## 任务 2：刷写触发器 (Flush Trigger)

在此任务中，您需要修改：

```
src/lsm_storage.rs
src/compact.rs
```

当内存中内存表（不可变 + 可变）的数量超过 LSM 存储选项中的 `num_memtable_limit` 时，您应该将最早的内存表刷写到磁盘。这是由后台的刷写线程完成的。刷写线程将随 `MiniLSM` 结构一起启动。我们已经实现了启动线程和正确停止线程的必要代码。

在此任务中，您需要在 `compact.rs` 中实现 `LsmStorageInner::trigger_flush`，并在 `lsm_storage.rs` 中实现 `MiniLsm::close`。`trigger_flush` 将每 50 毫秒执行一次。如果内存表的数量超过限制，您应该调用 `force_flush_next_imm_memtable` 来刷写一个内存表。当用户调用 `close` 函数时，您应该等待刷写线程（以及第 2 周中的压缩线程）完成。

## 任务 3：过滤 SST (Filter the SSTs)

现在您有了一个功能齐全的存储引擎，您可以使用 mini-lsm-cli 与您的存储引擎进行交互。

```shell
cargo run --bin mini-lsm-cli -- --compaction none
```

然后，

```
fill 1000 3000
get 2333
flush
fill 1000 3000
get 2333
flush
get 2333
scan 2000 2333
```

如果您填充更多数据，您可以看到刷写线程正在工作，并自动刷写 L0 SST，而无需使用 `flush` 命令。

最后，在本周结束之前，让我们实现一个简单的优化来过滤 SST。根据用户提供的键范围，我们可以轻松过滤掉一些不包含该键范围的 SST，这样我们就不需要在合并迭代器中读取它们。

在此任务中，您需要修改：

```
src/lsm_storage.rs
src/iterators/*
src/lsm_iterator.rs
```

您需要更改读取路径函数以跳过不可能包含键/键范围的 SST。您需要为迭代器实现 `num_active_iterators`，以便测试用例可以检查您的实现是否正确。对于 `MergeIterator` 和 `TwoMergeIterator`，它是所有子迭代器的 `num_active_iterators` 之和。请注意，如果您没有修改 `MergeIterator` 入门代码中的字段，请记住也要考虑 `MergeIterator::current`。对于 `LsmIterator` 和 `FusedIterator`，只需从内部迭代器返回活动迭代器的数量即可。

您可以实现类似 `range_overlap` 和 `key_within` 的辅助函数来简化您的代码。

## 测试您的理解 (Test Your Understanding)

* 如果用户请求删除一个键两次会发生什么？
* 初始化迭代器时，将同时加载多少内存（或多少块）到内存中？
* 一些疯狂的用户想要*分叉*(fork)他们的 LSM 树。他们希望启动引擎以摄取一些数据，然后对其进行分叉，以便获得两个相同的数据集，然后分别对它们进行操作。一种简单但效率不高的方法是简单地将所有 SST 和内存结构复制到一个新目录并启动引擎。但是，请注意，我们从不修改磁盘上的文件，实际上我们可以重用父引擎的 SST 文件。您认为如何才能在不复制数据的情况下有效地实现此分叉功能？（请查看 [Neon Branching](https://neon.tech/docs/introduction/branching)）。
* 假设您正在构建一个多租户 LSM 系统，您在一台具有 128GB 内存的机器上托管 1 万个数据库。内存表大小限制设置为 256MB。此设置需要多少内存表内存？
  * 显然，您没有足够的内存容纳所有这些内存表。假设每个用户仍然拥有自己的内存表，您将如何设计内存表刷写策略以使其正常工作？让所有这些用户共享同一个内存表（例如，通过将租户 ID 编码为键前缀）是否有意义？

我们不提供这些问题的参考答案，欢迎在 Discord 社区中讨论它们。

## 奖励任务 (Bonus Tasks)

* **实现写入/L0 暂停 (Implement Write/L0 Stall)。** 当内存表的数量过多地超过最大数量时，您可以阻止用户写入存储引擎。在第 2 周实现压缩后，您还可以为 L0 表实现写入暂停。
* **前缀扫描 (Prefix Scan)。** 您可以通过实现前缀扫描接口并使用前缀信息来过滤更多的 SST。

{{#include copyright.md}}
