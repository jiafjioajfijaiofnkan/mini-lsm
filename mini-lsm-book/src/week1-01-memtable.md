<!--
  mini-lsm-book © 2022-2025 by Alex Chi Z is licensed under CC BY-NC-SA 4.0
-->

# 内存表 (Memtables)

![本章概览](./lsm-tutorial/week1-01-overview.svg)

在本章中，您将：

* 基于跳表 (skiplist) 实现内存表 (memtable)。
* 实现冻结内存表的逻辑。
* 为内存表实现 LSM 读取路径 `get`。

要将测试用例复制到入门代码并运行它们：

```
cargo x copy-test --week 1 --day 1
cargo x scheck
```

## 任务 1：跳表内存表 (SkipList Memtable)

在此任务中，您需要修改：

```
src/mem_table.rs
```

首先，让我们实现 LSM 存储引擎的内存结构 —— 内存表 (memtable)。我们选择 [crossbeam 的跳表实现](https://docs.rs/crossbeam-skiplist/latest/crossbeam_skiplist/) 作为内存表的数据结构，因为它支持无锁并发读写。我们不会深入探讨跳表的工作原理，简而言之，它是一个有序的键值映射，可以轻松实现并发读写。

crossbeam-skiplist 提供了与 Rust 标准库的 `BTreeMap` 类似的接口：insert (插入)、get (获取) 和 iter (迭代)。唯一的区别是修改接口（例如 `insert`）只需要对跳表的不可变引用，而不是可变引用。因此，在您的实现中，实现内存表结构时不应获取任何互斥锁。

您还会注意到 `MemTable` 结构没有 `delete` (删除) 接口。在 mini-lsm 实现中，删除操作表示为一个键对应一个空值。

在此任务中，您需要实现 `MemTable::get` 和 `MemTable::put` 以启用对内存表的修改。请注意，如果键已存在，`put` 应始终覆盖该键。单个内存表中不会有相同键的多个条目。

我们使用 `bytes` crate 来存储内存表中的数据。`bytes::Byte` 类似于 `Arc<[u8]>`。当您克隆 `Bytes` 或获取 `Bytes` 的切片时，底层数据不会被复制，因此克隆操作的成本很低。相反，它只是创建一个对存储区域的新引用，当没有引用指向该区域时，存储区域将被释放。

## 任务 2：引擎中的单个内存表 (A Single Memtable in the Engine)

在此任务中，您需要修改：

```
src/lsm_storage.rs
```

现在，我们将第一个数据结构——内存表——添加到 LSM 状态中。在 `LsmStorageState::create` 中，您会发现当创建 LSM 结构时，我们将初始化一个 ID 为 0 的内存表。这是初始状态下的**可变内存表 (mutable memtable)**。在任何时候，引擎都只有一个可变内存表。内存表通常有一个大小限制（例如 256MB），当达到大小限制时，它将被冻结为不可变内存表。

查看 `lsm_storage.rs`，您会发现有两个结构表示存储引擎：`MiniLSM` 和 `LsmStorageInner`。`MiniLSM` 是 `LsmStorageInner` 的一个简单包装器。您将在 `LsmStorageInner` 中实现大部分功能，直到第二周的压缩部分。

`LsmStorageState` 存储 LSM 存储引擎的当前结构。目前，我们只使用 `memtable` 字段，它存储当前的可变内存表。在此任务中，您需要实现 `LsmStorageInner::get`、`LsmStorageInner::put` 和 `LsmStorageInner::delete`。所有这些都应直接将请求分派给当前的内存表。

![单个内存表的 LSM](./lsm-tutorial/week1-01-single.svg)

您的 `delete` 实现应该简单地为该键放置一个空切片，我们称之为*删除墓碑 (delete tombstone)*。您的 `get` 实现应相应地处理这种情况。

要访问内存表，您需要获取 `state` 锁。由于我们的内存表实现仅需要对 `put` 的不可变引用，因此您只需要获取 `state` 上的读锁即可修改内存表。这允许多个线程并发访问内存表。

## 任务 3：写入路径 - 冻结内存表 (Write Path - Freezing a Memtable)

在此任务中，您需要修改：

```
src/lsm_storage.rs
src/mem_table.rs
```

![单个内存表的 LSM](./lsm-tutorial/week1-01-frozen.svg)

内存表的大小不能持续增长，当它达到大小限制时，我们需要将其冻结（稍后刷写到磁盘）。您可以在 `LsmStorageOptions` 中找到内存表的大小限制，它**等于 SST (有序字符串表) 的大小限制**（而不是 `num_memtables_limit`）。这不是一个硬性限制，您应该尽力冻结内存表。

在此任务中，当在内存表中 put/delete 一个键时，您需要计算近似的内存表大小。这可以通过在调用 `put` 时简单地将键和值的总字节数相加来计算。如果一个键被 put 两次，尽管跳表只包含最新的值，但您可以在近似的内存表大小中计算两次。一旦内存表达到限制，您应该调用 `force_freeze_memtable` 来冻结内存表并创建一个新的内存表。

`LsmStorageInner` 中的 `state: Arc<RwLock<Arc<LsmStorageState>>>` 字段采用这种结构是为了并发且安全地管理 LSM 树的整体状态，主要使用写时复制 (Copy-on-Write, CoW) 策略：

1. 内部的 `Arc<LsmStorageState>`：这持有一个实际 `LsmStorageState`（包含内存表列表、SST 引用等）的**不可变快照**。克隆这个 `Arc` 的成本非常低（只是一个原子引用计数增量），并为任何读取器在其操作期间提供一致且不变的状态视图。

2. `RwLock<Arc<LsmStorageState>>`：此读写锁保护指向当前 `Arc<LsmStorageState>`（活动快照）的*指针*。
    * **读取器**获取读锁，克隆 `Arc<LsmStorageState>`（获取它们自己对当前快照的引用），然后快速释放读锁。然后它们可以在没有进一步锁定的情况下使用其快照。
    * **写入器**（在修改状态时，例如冻结内存表）将：
        * 创建一个新的 `LsmStorageState` 实例，通常是通过从当前快照克隆数据然后应用修改。
        * 将此新状态包装在一个新的 `Arc<LsmStorageState>` 中。
        * 获取 `RwLock` 上的写锁。
        * 用新的 `Arc<LsmStorageState>` 替换旧的。
        * 释放写锁。

3. 外部的 `Arc<RwLock<...>>`：这允许 `RwLock` 本身（以及因此访问和更新状态的机制）在可能需要与 `LsmStorageInner` 交互的多个线程或应用程序部分之间安全地共享。

这种 CoW 方法确保读取器始终看到有效、一致的状态快照，并经历最小的阻塞。写入器通过交换整个状态快照来原子地更新状态，从而减少了关键锁的持有时间，从而提高了并发性。

因为可能有多个线程将数据存入存储引擎，所以 `force_freeze_memtable` 可能会被多个线程并发调用。在这种情况下，您需要考虑如何避免竞争条件。

在很多地方您可能希望修改 LSM 状态：冻结可变内存表、将内存表刷写到 SST 以及 GC (垃圾回收)/压缩。在所有这些修改过程中，都可能存在 I/O 操作。一种直观的锁定策略结构是：

```rust,no_run
fn freeze_memtable(&self) {
    let state = self.state.write();
    state.immutable_memtable.push(/* 某些东西 */);
    state.memtable = MemTable::create();
}
```

...即您在 LSM 状态的写锁内修改所有内容。

目前这可以正常工作。但是，考虑一下您希望为您创建的每个内存表创建一个预写日志文件的情况。

```rust,no_run
fn freeze_memtable(&self) {
    let state = self.state.write();
    state.immutable_memtable.push(/* 某些东西 */);
    state.memtable = MemTable::create_with_wal()?; // <- 可能需要几毫秒
}
```

现在，当我们冻结内存表时，其他线程在几毫秒内无法访问 LSM 状态，这会导致延迟峰值。

为了解决这个问题，我们可以将 I/O 操作移到锁区域之外。

```rust,no_run
fn freeze_memtable(&self) {
    let memtable = MemTable::create_with_wal()?; // <- 可能需要几毫秒
    {
        let state = self.state.write();
        state.immutable_memtable.push(/* 某些东西 */);
        state.memtable = memtable;
    }
}
```

然后，我们的状态写锁区域内就没有耗时的操作了。现在，考虑内存表即将达到容量限制，并且两个线程成功地将两个键放入内存表，它们都在放入两个键后发现内存表达到容量限制的情况。它们都会对内存表进行大小检查并决定冻结它。在这种情况下，我们可能会创建一个空的内存表，然后立即将其冻结。

为了解决这个问题，所有状态修改都应通过状态锁进行同步。

```rust,no_run
fn put(&self, key: &[u8], value: &[u8]) {
    // 将内容放入内存表，检查容量，然后释放 LSM 状态的读锁
    if memtable_reaches_capacity_on_put {
        let state_lock = self.state_lock.lock();
        if /* 再次检查当前内存表是否达到容量 */ {
            self.freeze_memtable(&state_lock)?;
        }
    }
}
```

您会在未来的章节中经常看到这种模式。例如，对于 L0 刷写：

```rust,no_run
fn force_flush_next_imm_memtable(&self) {
    let state_lock = self.state_lock.lock();
    // 获取最旧的内存表并释放 LSM 状态的读锁
    // 将内容写入磁盘
    // 获取 LSM 状态的写锁并更新状态
}
```

这确保只有一个线程能够修改 LSM 状态，同时仍然允许对 LSM 存储进行并发访问。

在此任务中，您需要修改 `put` 和 `delete` 以遵循内存表的软容量限制。当达到限制时，调用 `force_freeze_memtable` 来冻结内存表。请注意，我们没有针对此并发场景的测试用例，您需要自行考虑所有可能的竞争条件。另外，请记住检查锁区域，以确保临界区是所需的最小范围。

您可以简单地将下一个内存表 ID 分配为 `self.next_sst_id()`。请注意，`imm_memtables` 按从最新到最早的顺序存储内存表。也就是说，`imm_memtables.first()` 应该是最后冻结的内存表。

## 任务 4：读取路径 - Get (Read Path - Get)

在此任务中，您需要修改：

```
src/lsm_storage.rs
```

既然您有多个内存表，您可以修改读取路径的 `get` 函数以获取键的最新版本。确保您按从最新到最早的顺序探测内存表。

## 测试您的理解 (Test Your Understanding)

* 为什么内存表不提供 `delete` API？
* 内存表存储所有写入操作而不是仅存储键的最新版本是否有意义？例如，用户将 a->1、a->2 和 a->3 放入同一个内存表。
* 是否可以使用其他数据结构作为 LSM 中的内存表？使用跳表的优缺点是什么？
* 为什么我们需要 `state` 和 `state_lock` 的组合？我们能只使用 `state.read()` 和 `state.write()` 吗？
* 为什么存储和探测内存表的顺序很重要？如果一个键出现在多个内存表中，您应该向用户返回哪个版本？
* 内存表的内存布局是否高效/是否具有良好的数据局部性？（想想 `Byte` 是如何实现并存储在跳表中的……）有哪些可能的优化可以使内存表更高效？
* 我们在本课程中使用 `parking_lot` 锁。它的读写锁是公平锁吗？如果有一个写入者正在等待现有读取者停止，那么尝试获取锁的读取者可能会发生什么情况？
* 冻结内存表后，是否可能某些线程仍然持有旧的 LSM 状态并写入这些不可变的内存表？您的解决方案如何防止这种情况发生？
* 在某些地方，您可能首先获取状态的读锁，然后释放它并获取写锁（这两个操作可能在不同的函数中，但由于一个函数调用另一个函数而顺序发生）。这与直接将读锁升级为写锁有何不同？是否有必要升级而不是获取和释放，升级的成本是多少？

我们不提供这些问题的参考答案，欢迎在 Discord 社区中讨论它们。

## 奖励任务 (Bonus Tasks)

* **更多内存表格式 (More Memtable Formats)。** 您可以实现其他内存表格式。例如，BTree 内存表、向量内存表和 ART 内存表。

{{#include copyright.md}}
