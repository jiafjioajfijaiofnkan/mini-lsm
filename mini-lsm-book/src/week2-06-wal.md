<!--
  mini-lsm-book © 2022-2025 by Alex Chi Z is licensed under CC BY-NC-SA 4.0
-->

# 预写日志 (Write-Ahead Log, WAL)

![本章概览](./lsm-tutorial/week2-06-overview.svg)

在本章中，您将：

* 实现预写日志文件的编码和解码。
* 在系统重启时从 WAL 恢复内存表。

要将测试用例复制到入门代码并运行它们：

```
cargo x copy-test --week 2 --day 6
cargo x scheck
```

## 任务 1：WAL 编码 (WAL Encoding)

在此任务中，您需要修改：

```
src/wal.rs
```

在上一章中，我们已经实现了 manifest 文件，以便 LSM 状态可以持久化。并且我们实现了 `close` 函数，以便在停止引擎之前将所有内存表刷写到 SST。现在，如果系统崩溃（例如，断电）会发生什么？我们可以将内存表修改记录到 WAL（预写日志），并在数据库重启时恢复 WAL。WAL 仅在 `self.options.enable_wal = true` 时启用。

WAL 编码只是一个键值对列表。

```
| key_len | key | value_len | value |
```

您还需要实现 `recover` 函数来读取 WAL 并恢复内存表的状态。

请注意，我们使用 `BufWriter` 来写入 WAL。使用 `BufWriter` 可以减少对操作系统的系统调用次数，从而降低写入路径的延迟。当用户修改键时，数据不保证立即写入磁盘。相反，引擎仅在调用 `sync` 时才保证数据已持久化。要正确地将数据持久化到磁盘，您需要首先通过调用 `flush()` 将数据从缓冲区写入器刷写到文件对象，然后通过使用 `get_mut().sync_all()` 对文件执行 fsync。请注意，您*仅*需要在引擎的 `sync` 被调用时执行 fsync。您*不*需要在每次写入数据时都执行 fsync。

## 任务 2：集成 WAL (Integrate WALs)

在此任务中，您需要修改：

```
src/mem_table.rs
src/wal.rs
src/lsm_storage.rs
```

`MemTable` 有一个 WAL 字段。如果 `wal` 字段设置为 `Some(wal)`，则在更新内存表时需要追加到 WAL。在您的 LSM 引擎中，如果 `enable_wal = true`，则需要创建 WAL。当创建新的内存表时，您还需要使用 `ManifestRecord::NewMemtable` 记录更新 manifest。

您可以使用 `create_with_wal` 函数创建带有 WAL 的内存表。WAL 应写入存储目录中的 `<memtable_id>.wal`。如果此内存表作为 L0 SST 刷写，则内存表 ID 应与 SST ID 相同。

## 任务 3：从 WAL 恢复 (Recover from the WALs)

在此任务中，您需要修改：

```
src/lsm_storage.rs
```

如果启用了 WAL，则在加载数据库时，您需要根据 WAL 恢复内存表。您还需要实现数据库的 `sync` 函数。`sync` 的基本保证是引擎确信数据已持久化到磁盘（并在重启时将恢复）。为实现此目的，您可以简单地同步与当前内存表对应的 WAL。

```
cargo run --bin mini-lsm-cli -- --enable-wal
```

请记住从状态中恢复正确的 `next_sst_id`，它应该是 `max{memtable id, sst id}` + 1。在您的 `close` 函数中，如果设置了 `enable_wal` 为 true，则不应将内存表刷写到 SST，因为 WAL 本身提供了持久性。在关闭数据库之前，您应该等待所有压缩和刷写线程退出。

## 测试您的理解 (Test Your Understanding)

* 您应该在引擎中的什么时候调用 `fsync`？如果您过于频繁地调用 `fsync`（例如，在每个 put key 请求时）会发生什么？
* 通常情况下，在 SSD（固态硬盘）上 `fsync` 操作的成本有多高？
* 您什么时候可以告诉用户他们的修改（put/delete）已持久化？
* 您如何处理 WAL 中的损坏数据？
* 是否可以设计一个没有 WAL 的 LSM 引擎（即，使用 L0 作为 WAL）？这种设计的含义是什么？

我们不提供这些问题的参考答案，欢迎在 Discord 社区中讨论它们。

{{#include copyright.md}}
