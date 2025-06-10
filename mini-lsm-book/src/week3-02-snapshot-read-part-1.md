<!--
  mini-lsm-book © 2022-2025 by Alex Chi Z is licensed under CC BY-NC-SA 4.0
-->

# 快照读取 - 内存表和时间戳 (Snapshot Read - Memtables and Timestamps)

在本章中，您将：

* 重构您的内存表/预写日志 (WAL) 以存储键的多个版本。
* 实现新的引擎写入路径，为每个键分配时间戳。
* 使您的压缩过程能够感知多版本键。
* 实现新的引擎读取路径以返回键的最新版本。

在重构过程中，您可能需要根据需要将某些函数的签名从 `&self` 更改为 `self: &Arc<Self>`。

要运行测试用例：

```
cargo x copy-test --week 3 --day 2
cargo x scheck
```

**注意：MVCC (多版本并发控制) 子系统直到第 3 周第 2 天才会完全实现。在这一天结束时，您只需要通过第 3 周第 2 天的测试以及所有 <= 2.4 的测试。**

## 任务 1：内存表、预写日志和读取路径 (MemTable, Write-Ahead Log, and Read Path)

在此任务中，您需要修改：

```
src/wal.rs
src/mem_table.rs
src/lsm_storage.rs
```

我们已经将引擎中的大部分键改为了 `KeySlice`，它包含一个字节键和一个时间戳。但是，我们系统的某些部分仍然没有考虑时间戳。在我们的第一个任务中，您需要修改您的内存表和 WAL 实现以考虑时间戳。

您首先需要更改存储在内存表中的 `SkipMap` 的类型。

```rust,no_run
pub struct MemTable {
    // map: Arc<SkipMap<Bytes, Bytes>>,
    map: Arc<SkipMap<KeyBytes, Bytes>>, // Bytes -> KeyBytes
    // ...
}
```

之后，您可以继续修复所有编译器错误以完成此任务。

**MemTable::get**

我们保留 get 接口，以便测试用例仍然可以探测内存表中键的特定版本。完成此任务后，此接口不应在您的读取路径中使用。鉴于我们在 skiplist 中存储 `KeyBytes`（即 `(Bytes, u64)`），而用户探测的是 `KeySlice`（即 `(&[u8], u64)`）。我们必须找到一种方法将后者转换为前者的引用，以便我们可以检索 skiplist 中的数据。

为此，您可以使用不安全代码强制将 `&[u8]` 转换为静态类型，并使用 `Bytes::from_static` 从静态切片创建字节对象。这是合理的，因为 `Bytes` 不会尝试释放切片的内存，因为它被假定为静态的。

<details>

<summary>剧透：将 u8 切片转换为 Bytes</summary>

```rust,no_run
Bytes::from_static(unsafe { std::mem::transmute(key.key_ref()) })
```

</details>

这以前不是问题，因为我们之前拥有的是 `Bytes` 和 `&[u8]`，其中 `Bytes` 实现了 `Borrow<[u8]>`。

**MemTable::put**

签名应更改为 `fn put(&self, key: KeySlice, value: &[u8])`，并且您需要在实现中将键切片转换为 `KeyBytes`。

**MemTable::scan**

签名应更改为 `fn scan(&self, lower: Bound<KeySlice>, upper: Bound<KeySlice>) -> MemTableIterator`。您需要将 `KeySlice` 转换为 `KeyBytes`，并将它们用作 `SkipMap::range` 参数。

**MemTable::flush**

现在，在将内存表刷写到 SST 时，您应该使用键的时间戳，而不是使用默认时间戳。

**MemTableIterator**

它现在应该存储 `(KeyBytes, Bytes)`，并且返回的键类型应该是 `KeySlice`。

**Wal::recover** 和 **Wal::put**

预写日志现在应该接受键切片而不是用户键切片。在序列化和反序列化 WAL 记录时，您应该将时间戳放入 WAL 文件，并对时间戳和您之前拥有的所有其他字段进行校验和计算。

WAL 格式如下：

```
| key_len (不包括 ts 长度) (u16) | key | ts (u64) | value_len (u16) | value | checksum (u32) |
```

**LsmStorageInner::get**

以前，我们将 `get` 实现为首先探测内存表，然后扫描 SST。现在我们将内存表更改为使用新的键-时间戳 API，我们需要重新实现 `get` 接口。最简单的方法是像您在 `scan` 中所做的那样，为我们拥有的所有内容（内存表、不可变内存表、L0 SST 和其他级别的 SST）创建一个合并迭代器，不同之处在于我们对 SST 进行布隆过滤器过滤。

**LsmStorageInner::scan**

您需要合并新的内存表 API，并且应将扫描范围设置为 `(user_key_begin, TS_RANGE_BEGIN)` 和 `(user_key_end, TS_RANGE_END)`。请注意，当您处理排除边界时，您需要正确地将迭代器定位到下一个键（而不是具有相同时间戳的当前键）。

## 任务 2：写入路径 (Write Path)

在此任务中，您需要修改：

```
src/lsm_storage.rs
```

我们在 `LsmStorageInner` 中有一个 `mvcc` 字段，其中包含本周多版本并发控制所需的所有数据结构。当您打开目录并初始化存储引擎时，您需要创建该结构。

在您的 `write_batch` 实现中，您需要为写入批处理中的所有键获取一个提交时间戳。您可以在逻辑开始时使用 `self.mvcc().latest_commit_ts() + 1` 获取时间戳，并在逻辑结束时使用 `self.mvcc().update_commit_ts(ts)` 来增加下一个提交时间戳。为确保所有写入批处理具有不同的时间戳，并且新键位于旧键之上，您需要在函数开始时持有写锁 `self.mvcc().write_lock.lock()`，以便一次只有一个线程可以写入存储引擎。

## 任务 3：MVCC 压缩 (MVCC Compaction)

在此任务中，您需要修改：

```
src/compact.rs
```

我们在前面章节中所做的是仅保留键的最新版本，并在将键压缩到最底层时（如果键已删除）移除该键。使用 MVCC 后，我们现在将时间戳与键关联起来，因此不能再使用相同的逻辑进行压缩。

在本章中，您可以简单地删除移除键的逻辑。现在可以忽略 `compact_to_bottom_level`，并且在压缩过程中应保留键的所有版本。

此外，您需要以这样一种方式实现压缩算法：即使超出了 SST 大小限制，具有不同时间戳的相同键也应放在同一个 SST 文件中。这确保了如果在某个级别的 SST 中找到了一个键，那么它就不会出现在该级别的其他 SST 文件中，从而简化了系统许多部分的实现。

## 任务 4：LSM 迭代器 (LSM Iterator)

在此任务中，您需要修改：

```
src/lsm_iterator.rs
```

在上一章中，我们实现了 LSM 迭代器，其行为是将具有不同时间戳的相同键视为不同的键。现在，我们需要重构 LSM 迭代器，以便在从子迭代器检索到键的多个版本时仅返回键的最新版本。

您需要在迭代器中记录 `prev_key`。如果我们已经向用户返回了键的最新版本，我们可以跳过所有旧版本并继续处理下一个键。

此时，您应该通过前面章节中的所有测试，持久性测试（2.5 和 2.6）除外。

## 测试您的理解 (Test Your Understanding)

* MVCC 引擎中的 `get` 与您在第 2 周构建的引擎中的 `get` 有何不同？
* 在第 2 周，当找到键时，您会在第一个内存表/级别停止 `get`。在 MVCC 版本中可以这样做吗？
* 您如何将 `KeySlice` 转换为 `&KeyBytes`？这是一个安全/合理的操作吗？
* 为什么我们需要在写入路径中获取写锁？

我们不提供这些问题的参考答案，欢迎在 Discord 社区中讨论它们。

## 奖励任务 (Bonus Tasks)

* **内存表 Get 提前停止 (Early Stop for Memtable Gets)。** 我们可以实现 `get` 如下：如果我们在内存表中找到了一个键的版本，就可以停止搜索，而不是为所有内存表和 SST 创建合并迭代器。这同样适用于 SST。

{{#include copyright.md}}
