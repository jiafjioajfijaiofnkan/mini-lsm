<!--
  mini-lsm-book © 2022-2025 by Alex Chi Z is licensed under CC BY-NC-SA 4.0
-->

# 存储引擎和块缓存 (Storage Engine and Block Cache)

<div class="warning">

这是 Mini-LSM 课程的旧版本，我们将不再维护它。我们现在有了这个课程的更好版本，本章内容现在是 [Mini-LSM 第 1 周第 5 天：读取路径](./week1-05-read-path.md) 和 [Mini-LSM 第 1 周第 6 天：写入路径](./week1-06-write-path.md) 的一部分。

</div>

<!-- toc -->

在这一部分，您需要修改：

* `src/lsm_iterator.rs`
* `src/lsm_storage.rs`
* `src/table.rs`
* 其他使用 `SsTable::read_block` 的部分

您可以使用 `cargo x copy-test day4` 将我们提供的测试用例复制到入门代码目录。完成此部分后，使用 `cargo x scheck` 检查样式并运行所有测试用例。如果您想编写自己的测试用例，请在 `table.rs` 中编写一个新的模块 `#[cfg(test)] mod user_tests { /* 您的测试用例 */ }`。请记住移除您修改的模块顶部的 `#![allow(...)]`，以便 cargo clippy 能够实际检查样式。

## 任务 1 - Put 和 Delete (Put and Delete)

在实现 put 和 delete 之前，让我们回顾一下 LSM 树的工作原理。LSM 的结构包括：

* MemTable (内存表)：一个活动的可变 memtable 和多个不可变 memtable。
* 预写日志 (Write-ahead log, WAL)：每个 memtable 对应一个 WAL。
* SST (有序字符串表)：memtable 可以以 SST 格式刷写到磁盘。SST 以多个层级组织。

在这一部分，我们只需要获取锁，并将条目（或墓碑, tombstone）写入活动的 memtable 中。您可以修改 `lsm_storage.rs`。

## 任务 2 - Get (获取)

要从 LSM 获取一个值，我们可以简单地从活动的 memtable、不可变的 memtable（从最新到最早）以及所有的 SST 中探测。为了减少临界区 (critical section)，我们可以持有读锁，将所有指向 memtable 和 SST 的指针复制出 `LsmStorageInner` 结构，并在临界区之外创建迭代器。在创建迭代器和探测时要小心顺序。

## 任务 3 - Scan (扫描)

要创建一个扫描迭代器 `LsmIterator`，您需要使用 `TwoMergeIterator` 来合并 memtable 上的 `MergeIterator` 和 SST 上的 `MergeIterator`。您可以在 `lsm_iterator.rs` 中实现这一点。可选地，您可以实现 `FusedIterator`，这样如果用户在迭代器失效后意外调用 `next`，底层迭代器就不会 panic。

`TwoMergeIterator` 生成的键值对序列可能包含空值，这意味着该值已被删除。`LsmIterator` 应过滤掉这些空值。此外，它还需要正确处理起始和结束边界。

## 任务 4 - Sync (同步)

在这一部分，我们将在 `lsm_storage.rs` 中实现 memtable 并将其刷写到 L0 SST。与任务 1 一样，写入操作直接进入活动的可变 memtable。一旦调用 `sync`，我们将分两步将 SST 刷写到磁盘：

* 首先，将当前可变的 memtable 移动到不可变 memtable 列表，这样未来的请求就不会进入当前的 memtable。创建一个新的 memtable。所有这些都应该在一个单独的临界区内发生，并阻塞所有读取。
* 然后，我们可以在不持有任何锁的情况下将 memtable 作为 SST 文件刷写到磁盘。
* 最后，在一个临界区内，移除该 memtable 并将 SST 放入 `l0_tables`。

一次只能有一个线程进行同步，因此您应该使用互斥锁 (mutex) 来确保此要求。

## 任务 5 - 块缓存 (Block Cache)

既然我们已经实现了 LSM 结构，我们就可以开始向磁盘写入一些东西了！之前在 `table.rs` 中，我们实现了一个 `FileObject` 结构，但没有向磁盘写入任何内容。在这个任务中，我们将更改实现，以便：

* `read` 将使用 `std::os::unix::fs::FileExt` 中的 `read_exact_at` 从磁盘读取，不进行任何缓存。
* 文件的大小应存储在结构内部，`size` 函数直接返回它。
* `create` 应将文件写入磁盘。通常您应该对该文件调用 `fsync`。但这会大大减慢单元测试的速度。因此，在第 6 天的恢复实现之前，我们不执行 fsync。
* `open` 在第 6 天的恢复实现之前保持未实现状态。

之后，我们可以在 `SsTable` 上实现一个新的 `read_block_cached` 函数，以便我们可以利用块缓存来处理读取请求。在初始化 `LsmStorage` 结构时，您应该使用 `moka-rs` 创建一个大小为 4GB 的块缓存。块按 SST ID + 块 ID进行缓存。使用 `try_get_with` 从缓存中获取块/在缓存未命中时填充缓存。如果有多个请求读取同一个块并且缓存未命中，`try_get_with` 将只向磁盘发出单个读取请求，并将结果广播给所有请求。

请记住更改 `SsTableIterator` 以使用块缓存。

## 额外任务 (Extra Tasks)

* 正如您可能已经看到的，每次我们执行 get、put 或 delete 操作时，都需要获取保护 LSM 结构的读锁；如果我们想刷写，则需要获取写锁。这可能会导致很多问题。一些锁实现是公平的，这意味着只要有写者在等待锁，任何读者都无法获取锁。因此，写者将等待最慢的读者完成其操作，然后才能实际执行工作。一种可能的优化是实现 `WriteBatch`。我们不需要立即将用户的请求写入 memtable + WAL。我们可以允许用户进行批量写入。
* 将块对齐到 4K 并使用直接 I/O (direct I/O)。

{{#include copyright.md}}
