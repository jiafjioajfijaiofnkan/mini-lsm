<!--
  mini-lsm-book © 2022-2025 by Alex Chi Z is licensed under CC BY-NC-SA 4.0
-->

# Manifest (清单)

![本章概览](./lsm-tutorial/week2-05-overview.svg)

在本章中，您将：

* 实现 manifest 文件的编码和解码。
* 在系统重启时从 manifest 文件恢复。

要将测试用例复制到入门代码并运行它们：

```
cargo x copy-test --week 2 --day 5
cargo x scheck
```

## 任务 1：Manifest 编码 (Manifest Encoding)

系统使用 manifest 文件记录引擎中发生的所有操作。目前，只有两种类型的操作：压缩 (compaction) 和 SST (有序字符串表) 刷写 (flush)。当引擎重启时，它将读取 manifest 文件，重建状态，并加载磁盘上的 SST 文件。

存储 LSM 状态的方法有很多。最简单的方法之一是简单地将完整状态存储到一个 JSON 文件中。每次我们进行压缩或刷写新的 SST 时，都可以将整个 LSM 状态序列化到一个文件中。这种方法的问题在于，当数据库变得非常大（例如，1 万个 SST）时，将 manifest 写入磁盘会非常慢。因此，我们将 manifest 设计为仅追加 (append-only) 文件。

在此任务中，您需要修改：

```
src/manifest.rs
```

我们使用 JSON 编码 manifest 记录。您可以使用 `serde_json::to_vec` 将 manifest 记录编码为 JSON，将其写入 manifest 文件，然后执行 fsync。当您从 manifest 文件读取时，可以使用 `serde_json::Deserializer::from_slice`，它将返回一个记录流。您不需要存储记录长度等信息，因为 `serde_json` 可以自动找到记录的分割点。


manifest 格式如下：

```
| JSON 记录 | JSON 记录 | JSON 记录 | JSON 记录 |
```

再次注意，我们不记录每个记录有多少字节的信息。

引擎运行数小时后，manifest 文件可能会变得非常大。届时，您可以定期压缩 manifest 文件以存储当前快照并截断日志。您可以将此作为奖励任务来实现。


## 任务 2：写入 Manifest (Write Manifests)

现在您可以继续修改 LSM 引擎，以便在必要时写入 manifest。在此任务中，您需要修改：

```
src/lsm_storage.rs
src/compact.rs
```

目前，我们只使用两种类型的 manifest 记录：SST 刷写和压缩。SST 刷写记录存储刷写到磁盘的 SST ID。压缩记录存储压缩任务和生成的 SST ID。每次向磁盘写入新文件时，首先同步文件和存储目录，然后写入 manifest 并同步 manifest。manifest 文件应写入 `<path>/MANIFEST`。

要同步目录，您可以实现 `sync_dir` 函数，在其中可以使用 `File::open(dir).sync_all()?` 来同步它。在 Linux 上，目录是一个包含目录中文件列表的文件。通过对目录执行 fsync，您可以确保在断电的情况下，新写入（或删除）的文件对用户可见。

请记住，在后台压缩触发器（分层/简单/通用）和用户请求执行强制压缩时，都要写入压缩 manifest 记录。

## 任务 3：关闭时刷写 (Flush on Close)

在此任务中，您需要修改：

```
src/lsm_storage.rs
```

您需要实现 `close` 函数。如果 `self.options.enable_wal = false`（我们将在下一章介绍 WAL），则应在停止存储引擎之前将所有内存表刷写到磁盘，以便持久化所有用户更改。

## 任务 4：从状态恢复 (Recover from the State)

在此任务中，您需要修改：

```
src/lsm_storage.rs
```

现在，您可以修改 `open` 函数以从 manifest 文件恢复引擎状态。要恢复它，您需要首先生成需要加载的 SST 列表。您可以通过调用 `apply_compaction_result` 并恢复 LSM 状态中的 SST ID 来实现此目的。之后，您可以遍历状态并加载所有 SST（更新 sstables 哈希映射）。在此过程中，您需要计算最大 SST ID 并更新 `next_sst_id` 字段。之后，您可以使用该 ID 创建一个新的内存表，并将 ID 加一。

如果您已经实现了分层压缩，则可能在每次应用压缩结果时都对 SST 进行了排序。但是，使用 manifest 恢复时，您的排序逻辑将被破坏，因为在恢复过程中，您无法知道每个 SST 的起始键和结束键。为了解决这个问题，您需要读取 `apply_compaction_result` 函数的 `in_recovery` 标志。在恢复过程中，您不应尝试检索 SST 的第一个键。在 LSM 状态恢复并且所有 SST 都打开后，您可以在恢复过程结束时进行排序。

可选地，您可以在 manifest 中包含每个 SST 的起始键和结束键。RocksDB/BadgerDB 中使用了此策略，这样您就不需要在压缩应用过程中区分恢复模式和正常模式。

您可以使用 mini-lsm-cli 测试您的实现。

```
cargo run --bin mini-lsm-cli
fill 1000 2000
close
cargo run --bin mini-lsm-cli
get 1500
```

## 测试您的理解 (Test Your Understanding)

* 您什么时候需要调用 `fsync`？为什么需要对目录执行 fsync？
* 您需要在哪些地方写入 manifest？
* 考虑一个不使用 manifest 文件的 LSM 引擎的替代实现。相反，它在每个文件的头部记录级别/层信息，每次重新启动时扫描存储目录，并仅从目录中存在的文件恢复 LSM 状态。在这种实现中，是否可以正确维护 LSM 状态，以及可能存在哪些问题/挑战？
* 目前，我们在创建合并迭代器之前创建所有 SST/连接迭代器，这意味着在开始扫描过程之前，我们必须将所有级别的第一个 SST 的第一个块加载到内存中。我们在 manifest 中有起始/结束键，是否可以利用此信息延迟数据块的加载并加快返回第一个键值对的时间？
* 是否可以不在 manifest 中存储层/级别信息？即，我们只在 manifest 中存储我们拥有的 SST 列表，而不存储级别信息，并使用键范围和时间戳信息（SST 元数据）重建层/级别。

## 奖励任务 (Bonus Tasks)

* **Manifest 压缩 (Manifest Compaction)。** 当 manifest 文件中的日志数量过多时，您可以重写 manifest 文件以仅存储当前快照，并将新日志附加到该文件。
* **并行打开 (Parallel Open)。** 在收集要打开的 SST 列表后，您可以并行打开和解码它们，而不是逐个进行，从而加快恢复过程。

{{#include copyright.md}}
