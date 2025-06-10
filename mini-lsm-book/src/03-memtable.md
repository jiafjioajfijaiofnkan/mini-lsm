<!--
  mini-lsm-book © 2022-2025 by Alex Chi Z is licensed under CC BY-NC-SA 4.0
-->

# MemTable (内存表) 和合并迭代器 (Merge Iterators)

<div class="warning">

这是 Mini-LSM 课程的旧版本，我们将不再维护它。我们现在有了这个课程的更好版本，本章内容现在是 [Mini-LSM 第 1 周第 1 天：Memtable](./week1-01-memtable.md) 和 [Mini-LSM 第 1 周第 2 天：合并迭代器](./week1-02-merge-iterator.md) 的一部分。

</div>

<!-- toc -->

在这一部分，您需要修改：

* `src/iterators/merge_iterator.rs`
* `src/iterators/two_merge_iterator.rs`
* `src/mem_table.rs`

您可以使用 `cargo x copy-test day3` 将我们提供的测试用例复制到入门代码目录。完成此部分后，使用 `cargo x scheck` 检查样式并运行所有测试用例。如果您想编写自己的测试用例，请在 `table.rs` 中编写一个新的模块 `#[cfg(test)] mod user_tests { /* 您的测试用例 */ }`。请记住移除您修改的模块顶部的 `#![allow(...)]`，以便 cargo clippy 能够实际检查样式。

这是 LSM 树基本构建块的最后一部分。实现合并迭代器后，我们可以轻松地合并来自数据结构不同部分（memtable + SST）的数据，并获得一个遍历所有数据的迭代器。在第 4 部分中，我们将把所有这些组合起来，构建一个真正的存储引擎。

## 任务 1 - MemTable (内存表)

在本课程中，我们使用 [crossbeam-skiplist](https://docs.rs/crossbeam-skiplist) 作为 memtable 的实现。Skiplist (跳表) 类似于链表，数据存储在列表节点中，并且不会在内存中移动。与使用单个指针指向下一个元素不同，skiplist 中的节点包含多个指针，允许用户“跳过某些元素”，从而实现 `O(log n)` 的搜索、插入和删除。

在存储引擎中，用户将创建数据结构的迭代器。通常，一旦用户修改了数据结构，迭代器就会失效（C++ STL 和 Rust 容器就是这种情况）。然而，skiplist 允许我们同时访问和修改数据结构，因此在并发访问时可能会提高性能。有一些论文认为 skiplist 不好，但是数据在内存中保持原位的良好特性可以使我们的实现更容易。

在 `mem_table.rs` 中，您需要实现一个基于 crossbeam-skiplist 的 memtable。请注意，memtable 仅支持 `get`、`scan` 和 `put`，不支持 `delete`。删除操作表示为一个墓碑 (tombstone) `key -> empty value`，实际数据将在压缩过程（第 5 天）中删除。请注意，所有 `get`、`scan`、`put` 函数都只需要 `&self`，这意味着我们可以并发调用这些操作。

## 任务 2 - MemTable 迭代器 (MemTable Iterator)

现在您可以为 `MemTable` 实现一个迭代器 `MemTableIterator`。`memtable.iter(start, end)` 将创建一个迭代器，返回 `start` 和 `end` 范围内的所有元素。这里，start 是 `std::ops::Bound`，它包含 3 个变体：`Unbounded`、`Included(key)`、`Excluded(key)`。`std::ops::Bound` 的表达能力消除了记住 API 是闭区间还是开区间的需要。

请注意，`crossbeam-skiplist` 的迭代器具有与 skiplist 本身相同的生命周期，这意味着在使用迭代器时我们总是需要提供一个生命周期。这很难使用。您可以使用 `ouroboros` crate 创建一个自引用结构来消除生命周期。您会发现 [ouroboros 示例][ouroboros-example] 很有帮助。

[ouroboros-example]: https://github.com/joshua-maros/ouroboros/blob/main/examples/src/ok_tests.rs

```rust
pub struct MemTableIterator {
    /// 持有对 skiplist 的引用，以确保迭代器有效。
    map: Arc<SkipList>
    /// 然后迭代器的生命周期应与 `MemTableIterator` 结构本身的生命周期相同
    iter: SkipList::Iter<'this>
}
```

您还需要将 Rust 风格的迭代器 API 转换为我们的存储迭代器。在 Rust 中，我们使用 `next() -> Data`。但在本课程中，`next` 没有返回值，数据应通过 `key()` 和 `value()` 获取。您需要考虑一种实现方法。

<details>
<summary>剧透：MemTableIterator 结构</summary>

```rust
#[self_referencing]
pub struct MemTableIterator {
    map: Arc<SkipMap<Bytes, Bytes>>,
    #[borrows(map)]
    #[not_covariant]
    iter: SkipMapRangeIter<'this>,
    item: (Bytes, Bytes),
}
```

我们有 `map` 作为对 skiplist 的引用，`iter` 作为结构的自引用项，`item` 作为迭代器的最后一个项。您可能想过使用类似 `iter::Peekable` 的东西，但它在检索键和值时需要 `&mut self`。因此，一种方法是 (1) 在初始化 `MemTableIterator` 时从迭代器获取元素，并将其存储在 `item` 中 (2) 调用 `next` 时，我们从内部迭代器的 `next` 获取元素，并将内部迭代器移动到下一个位置。

</details>

在这个设计中，您可能已经注意到，只要我们拥有迭代器对象，memtable 就无法从内存中释放。在本课程中，我们假设用户操作是短暂的，因此这不会导致大问题。有关可能的改进，请参阅额外任务。

您也可以考虑使用 [AgateDB 的 skiplist](https://github.com/tikv/agatedb/tree/master/skiplist) 实现，它避免了创建自引用结构的问题。

## 任务 3 - 合并迭代器 (Merge Iterator)

既然您有很多 memtable 和 SST，您可能希望合并它们以获取键的最新出现。在 `merge_iterator.rs` 中，我们有 `MergeIterator`，它是一个合并所有*相同类型*迭代器的迭代器。在 `new` 函数中索引位置较低的迭代器具有更高的优先级，也就是说，如果我们有：

```
iter1: 1->a, 2->b, 3->c
iter2: 1->d
iter: MergeIterator::create(vec![iter1, iter2])
```

最终的迭代器将产生 `1->a, 2->b, 3->c`。iter1 中的数据将覆盖其他迭代器中的数据。

您可以使用 `BinaryHeap` (二叉堆) 来实现此合并迭代器。请注意，您永远不应将任何无效的迭代器放入二叉堆中。一个常见的陷阱是错误处理。例如，

```rust
let Some(mut inner_iter) = self.iters.peek_mut() {
    inner_iter.next()?; // <- 会导致问题
}
```

如果 `next` 返回错误（例如，由于磁盘故障、网络故障、校验和错误等），则它不再有效。然而，当我们跳出 if 条件并将错误返回给调用者时，`PeekMut` 的 drop 会尝试在堆中移动元素，这会导致访问无效的迭代器。因此，您需要自己处理所有错误，而不是在 `PeekMut` 的作用域内使用 `?`。

您还需要为存储迭代器定义一个包装器，以便 `BinaryHeap` 可以在所有迭代器之间进行比较。

## 任务 4 - 双路合并迭代器 (Two Merge Iterator)

LSM 有两种存储数据的结构：内存中的 memtable 和磁盘上的 SST。在我们分别为所有 SST 和所有 memtable 构建了迭代器之后，我们将需要一个新的迭代器来合并两种不同类型的迭代器。这就是 `TwoMergeIterator`。

您可以在 `two_merge_iter.rs` 中实现 `TwoMergeIterator`。与 `MergeIterator` 类似，如果两个迭代器中都找到相同的键，则第一个迭代器优先。

在本课程中，我们明确没有使用类似 `Box<dyn StorageIter>` 的东西来避免动态分派 (dynamic dispatch)。这是 LSM 存储引擎中常见的优化。

## 额外任务 (Extra Tasks)

* 实现不同的 memtable，看看它与 skiplist 有何不同。例如，BTree memtable。您会注意到，在不持有与迭代器相同时间跨度的锁的情况下，很难获得 B+ 树的迭代器。您可能需要考虑一些巧妙的方法来解决这个问题。
* 异步迭代器 (Async iterator)。一个有趣的探索是看看是否有可能异步化存储引擎中的所有内容。您可能会遇到一些与生命周期相关的问题，并需要设法解决它们。
* 前台迭代器 (Foreground iterator)。在本课程中，我们假设所有操作都是短暂的，因此我们可以在迭代器中持有对 memtable 的引用。如果用户长时间持有迭代器，整个 memtable（可能为 256MB）即使已刷写到磁盘，仍会保留在内存中。为了解决这个问题，我们可以向用户提供一个 `ForegroundIterator` / `LongIterator`。该迭代器将定期创建新的底层存储迭代器，以允许垃圾回收资源。

{{#include copyright.md}}
