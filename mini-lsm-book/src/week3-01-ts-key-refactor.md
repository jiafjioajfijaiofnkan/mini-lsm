<!--
  mini-lsm-book © 2022-2025 by Alex Chi Z is licensed under CC BY-NC-SA 4.0
-->

# 时间戳键编码 + 重构 (Timestamp Key Encoding + Refactor)

在本章中，您将：

* 重构您的实现以使用 键+时间戳 (key+ts) 表示。
* 使您的代码能够使用新的键表示进行编译。

要运行测试用例：

```
cargo x copy-test --week 3 --day 1
cargo x scheck
```

**注意：MVCC (多版本并发控制) 子系统直到第 3 周第 2 天才会完全实现。在这一天结束时，您只需要通过第 3 周第 1 天的测试和所有第 1 周的测试。由于压缩的原因，第 2 周的测试将无法工作。**

## 任务 0：使用 MVCC 键编码 (Use MVCC Key Encoding)

您需要将键编码模块替换为 MVCC 的模块。我们从原始键模块中删除了一些接口，并为键实现了新的比较器。如果您遵循了前面章节中的说明，并且没有对键使用 `into_inner`，那么在完成所有重构后，您应该能够通过第 3 天的所有测试用例。否则，您需要仔细检查那些只比较键而不查看时间戳的地方。

具体来说，键类型的定义已从：

```rust,no_run
pub struct Key<T: AsRef<[u8]>>(T);
```

...更改为：

```rust,no_run
pub struct Key<T: AsRef<[u8]>>(T /* 用户键 */, u64 /* 时间戳 */);
```

...其中键关联了一个时间戳。我们仅在系统内部使用此键表示。在用户界面方面，我们不要求用户提供时间戳，因此某些结构在引擎中仍使用 `&[u8]` 而不是 `KeySlice`。我们稍后将介绍需要更改函数签名的那些地方。现在，您只需要运行：

```
cp mini-lsm-mvcc/src/key.rs mini-lsm-starter/src/
```

还有其他存储时间戳的方法。例如，我们仍然可以使用 `pub struct Key<T: AsRef<[u8]>>(T);` 表示，但假设键的最后 8 个字节是时间戳。您也可以将其作为奖励任务的一部分来实现。

```plaintext
备选键表示：| user_key (变长) | ts (8 字节) | 在单个切片中
我们的键表示：| user_key 切片 | ts (u64) |
```

在 键+时间戳 编码中，具有最小用户键和最大时间戳的键将首先排序。例如，

```
("a", 233) < ("a", 0) < ("b", 233) < ("b", 0)
```

## 任务 1：在块中编码时间戳 (Encode Timestamps in Blocks)

您会注意到的第一件事是，替换键模块后，您的代码可能无法编译。在本章中，您需要做的就是使其能够编译。在此任务中，您需要修改：

```
src/block.rs
src/block/builder.rs
src/block/iterator.rs
```

您会注意到 `raw_ref()` 和 `len()` 已从键 API 中移除。取而代之的是，我们有 `key_ref` 来检索用户键的切片，以及 `key_len` 来检索用户键的长度。您需要重构块构建器和解码实现以使用新的 API。此外，您还需要更改块编码以对时间戳进行编码。在 `BlockBuilder::add` 中，您应该这样做。新的块条目记录将如下所示：


```
key_overlap_len (u16) | remaining_key_len (u16) | key (remaining_key_len) | timestamp (u64)
```

您可以使用 `raw_len` 来估计键所需的空间，并将时间戳存储在用户键之后。

更改块编码后，您需要相应地更改 `block.rs` 和 `iterator.rs` 中的解码。

## 任务 2：在 SST 中编码时间戳 (Encoding Timestamps in SSTs)

然后，您可以继续修改表格式：

```
src/table.rs
src/table/builder.rs
src/table/iterator.rs
```

具体来说，您需要更改块元数据编码以包含键的时间戳。所有其他代码保持不变。由于我们在所有函数的签名中都使用 `KeySlice`（例如，seek、add），新的键比较器应自动按用户键和时间戳对键进行排序。

在您的表构建器中，您可以直接使用 `key_ref()` 来构建布隆过滤器。这自然会为您的 SST 创建一个前缀布隆过滤器。

## 任务 3：LSM 迭代器 (LSM Iterators)

由于我们使用关联泛型类型使我们的大多数迭代器适用于不同的键类型（即 `&[u8]` 和 `KeySlice<'_>`），因此如果它们实现正确，我们不需要修改合并迭代器和连接迭代器。`LsmIterator` 是我们从内部键表示中剥离时间戳并将键的最新版本返回给用户的地方。在此任务中，您需要修改：

```
src/lsm_iterator.rs
```

目前，我们不修改 `LsmIterator` 的逻辑以仅保留键的最新版本。我们只是通过在将键传递给内部迭代器时向用户键附加时间戳，并在返回给用户时从键中剥离时间戳来使其能够编译。您的 LSM 迭代器现在的行为应该是向用户返回同一键的多个版本。

## 任务 4：内存表 (Memtable)

目前，我们保留内存表的逻辑。我们向用户返回一个键切片，并使用 `TS_DEFAULT` 刷写 SST。我们将在下一章中将内存表更改为 MVCC。在此任务中，您需要修改：

```
src/mem_table.rs
```

## 任务 5：引擎读取路径 (Engine Read Path)

在此任务中，您需要修改：

```
src/lsm_storage.rs
```

既然键中有了时间戳，在创建迭代器时，我们需要使用带有时间戳的键进行寻址，而不仅仅是用户键。您可以使用 `TS_RANGE_BEGIN`（即最大时间戳）创建一个键切片。

当您检查用户键是否在表中时，您可以简单地比较用户键，而无需比较时间戳。

此时，您应该构建您的实现并通过所有第 1 周的测试用例。系统中存储的所有键都将使用 `TS_DEFAULT`（即时间戳 0）。我们将在接下来的两章中使引擎完全支持多版本并通过所有测试用例。

{{#include copyright.md}}
