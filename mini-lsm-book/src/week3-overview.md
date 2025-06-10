<!--
  mini-lsm-book © 2022-2025 by Alex Chi Z is licensed under CC BY-NC-SA 4.0
-->

# 第 3 周概览：多版本并发控制 (Week 3 Overview: Multi-Version Concurrency Control)

在这一部分，您将在前两周构建的 LSM 引擎之上实现 MVCC (多版本并发控制)。我们将在键中添加时间戳编码以维护键的多个版本，并更改引擎的某些部分，以确保根据是否有用户正在读取旧版本来保留或垃圾回收旧数据。

本课程 MVCC 部分的总体方法受到 [BadgerDB](https://github.com/dgraph-io/badger) 的启发并部分基于它。

MVCC 的关键在于在存储引擎中存储和访问键的多个版本。因此，我们需要将键格式更改为 `用户键 + 时间戳 (u64)`。在用户界面方面，我们需要新的 API 来帮助用户访问历史版本。总之，我们将向键添加一个单调递增的时间戳。

在前面的部分中，我们假设较新的键位于 LSM 树的较上层，而较旧的键位于 LSM 树的较低层。在压缩过程中，如果发现多个版本的键，我们只保留最新版本，并且压缩过程通过仅合并相邻的层/级来确保较新的键将保留在较上层。在 MVCC 实现中，具有较大时间戳的键是最新的键。在压缩过程中，只有在没有用户访问数据库的旧版本时，我们才能删除该键。尽管不在上层保留键的最新版本可能仍会为 MVCC LSM 实现产生正确的结果，但在本课程中，我们选择保持该不变量，如果一个键有多个版本，则较新的版本将始终出现在较上层。

通常，有两种使用具有 MVCC 支持的存储引擎的方法。如果用户将引擎用作独立组件并且不想手动分配键的时间戳，他们将使用事务 API 从存储引擎存储和检索数据。时间戳对用户是透明的。另一种方法是将存储引擎集成到系统中，用户自行管理时间戳。为了比较这两种方法，我们可以查看它们提供的 API。我们使用 BadgerDB 的术语来描述这两种用法：隐藏时间戳的称为*非托管模式 (un-managed mode)*，而给予用户完全控制权的称为*托管模式 (managed mode)*。

**托管模式 API (Managed Mode APIs)**
```
get(key, read_timestamp) -> (value, write_timestamp)
scan(key_range, read_timestamp) -> iterator<key, value, write_timestamp>
put/delete/write_batch(key, timestamp)
set_watermark(timestamp) # 我们很快会讨论水印！
```

**非托管/普通模式 API (Un-managed/Normal Mode APIs)**
```
get(key) -> value
scan(key_range) -> iterator<key, value>
start_transaction() -> txn
txn.put/delete/write_batch(key, timestamp)
```

如您所见，托管模式 API 要求用户在执行操作时提供时间戳。时间戳可能来自某些集中的时间戳系统，或来自其他系统的日志（例如，Postgres 逻辑复制日志）。用户需要指定一个水印 (watermark)，低于该水印的版本可以被引擎删除。

而对于非托管 API，它与我们之前实现的相同，只是用户需要通过创建事务来写入和读取数据。当用户创建事务时，他们可以获得数据库的一致状态（即快照, snapshot）。即使其他线程/事务向数据库写入数据，这些数据对于正在进行的事务也是不可见的。存储引擎在内部管理时间戳，并且不向用户公开它们。

在本周，我们将首先花 3 天时间对表格式和内存表进行重构。我们会将键格式更改为键切片和时间戳。之后，我们将实现必要的 API 以提供一致的快照和事务。

这部分共有 7 个章节（天）：


* [第 1 天：时间戳键重构](./week3-01-ts-key-refactor.md)。您将把 `key` 模块更改为 MVCC 版本，并重构您的系统以使用带有时间戳的键。
* [第 2 天：快照读取 - 内存表和时间戳](./week3-02-snapshot-read-part-1.md)。您将重构内存表和写入路径以支持多版本读/写。
* [第 3 天：快照读取 - 事务 API](./week3-03-snapshot-read-part-2.md)。您将实现事务 API 并完成读取/写入路径的其余部分以支持快照读取。
* [第 4 天：水印和垃圾回收](./week3-04-watermark.md)。您将实现水印计算算法，并在压缩时实现垃圾回收以删除旧版本。
* [第 5 天：事务和乐观并发控制](./week3-05-txn-occ.md)。您将为所有事务创建一个私有工作区，并批量提交它们，以便一个事务的修改对其他事务不可见。
* [第 6 天：可串行化快照隔离](./week3-06-serializable.md)。您将实现 OCC (乐观并发控制) 可串行化检查，以确保对数据库的修改是可串行化的，并中止违反可串行性的事务。
* [第 7 天：压缩过滤器](./week3-07-compaction-filter.md)。在本周末，我们将把压缩时垃圾回收逻辑推广为压缩过滤器，以便根据用户要求在压缩时删除数据。

{{#include copyright.md}}
