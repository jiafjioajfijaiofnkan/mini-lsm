<!--
  mini-lsm-book © 2022-2025 by Alex Chi Z is licensed under CC BY-NC-SA 4.0
-->

# 合并迭代器 (Merge Iterator)

![本章概览](./lsm-tutorial/week1-02-overview.svg)

在本章中，您将：

* 实现内存表迭代器 (memtable iterator)。
* 实现合并迭代器 (merge iterator)。
* 为内存表实现 LSM 读取路径 `scan`。

要将测试用例复制到入门代码并运行它们：

```
cargo x copy-test --week 1 --day 2
cargo x scheck
```

## 任务 1：内存表迭代器 (Memtable Iterator)

在本章中，我们将实现 LSM 的 `scan` 接口。`scan` 使用迭代器 API 按顺序返回一定范围内的键值对。在上一章中，您已经实现了 `get` API 和创建不可变内存表的逻辑，您的 LSM 状态现在应该包含多个内存表。您需要首先在单个内存表上创建迭代器，然后在所有内存表上创建合并迭代器，最后为迭代器实现范围限制。

在此任务中，您需要修改：

```
src/mem_table.rs
```

所有 LSM 迭代器都实现了 `StorageIterator` trait。它有 4 个函数：`key`、`value`、`next` 和 `is_valid`。如果您熟悉 Rust 标准库的 `Iterator` trait，您可能会发现 `StorageIterator` 有些不同。相反，`StorageIterator` 采用了基于游标 (cursor-based) 的 API，这是一种在数据库系统中常见的设计模式，尤其受到 RocksDB 迭代器的启发（有关参考，请参见 [`iterator_base.h`](https://github.com/facebook/rocksdb/blob/main/include/rocksdb/iterator_base.h) 和 [`iterator.h`](https://github.com/facebook/rocksdb/blob/main/include/rocksdb/iterator.h)）。

创建迭代器时，其游标将停在某个元素上，`key` / `value` 将返回内存表/块/SST 中满足起始条件（即起始键）的第一个键。这两个接口将返回一个 `&[u8]` 以避免复制。

从调用者的角度来看，典型的使用模式是：

```rust
let mut iter: impl StorageIterator = ...;
while iter.is_valid() {
    let key = iter.key();
    let value = iter.value();
    // 处理键和值
    iter.next()?; // 前进到下一个项目，处理潜在的错误
}
```

`StorageIterator` 的核心方法具有独特的语义：

* `next()`：此方法仅负责尝试将游标移动到下一个元素。它返回一个 `Result` 以报告在此前进过程中遇到的任何错误（例如，I/O 问题）。它本身并*不*保证新位置有效，仅保证已尝试移动。
* `is_valid()`：此方法指示迭代器的当前游标是否指向有效的数据元素。它*不*会前进迭代器。

因此，作为 `StorageIterator` 的实现者，在每次调用 `next()` 之后（即使它在 `next()` 操作本身没有错误的情况下成功），您都有责任更新内部状态，以便 `is_valid()` 正确反映新的游标位置是否实际指向有效项。

总之，`next` 将游标移动到下一个位置。`is_valid` 返回迭代器是否已到达末尾或出错。您可以假设只有在 `is_valid` 返回 true 时才会调用 `next`。将会有一个 `FusedIterator` 包装器用于迭代器，以阻止在迭代器无效时调用 `next`，从而避免用户误用迭代器。

回到内存表迭代器。您应该已经发现迭代器没有任何与之关联的生命周期。想象一下，您创建一个 `Vec<u64>` 并调用 `vec.iter()`，迭代器类型将类似于 `VecIterator<'a>`，其中 `'a` 是 `vec` 对象的生命周期。这同样适用于 `SkipMap`，其中其 `iter` API 返回一个具有生命周期的迭代器。然而，在我们的例子中，我们不希望迭代器上具有此类生命周期，以避免使系统过于复杂（并且难以编译……）。

如果迭代器没有生命周期泛型参数，我们应该确保*无论何时使用迭代器，底层的 skiplist 对象都不会被释放*。实现这一点的唯一方法是将 `Arc<SkipMap>` 对象放入迭代器本身。要定义这样的结构：

```rust,no_run
pub struct MemtableIterator {
    map: Arc<SkipMap<Bytes, Bytes>>,
    iter: SkipMapRangeIter<'???'>,
}
```

好了，问题来了：我们希望表达迭代器的生命周期与结构中的 `map` 相同。我们该怎么做呢？

这是您在本课程中遇到的第一个也是最棘手的 Rust 语言问题 —— 自引用结构 (self-referential structure)。如果可以写成这样：

```rust,no_run
pub struct MemtableIterator { // <- 带有生命周期 'this
    map: Arc<SkipMap<Bytes, Bytes>>,
    iter: SkipMapRangeIter<'this>,
}
```

那么问题就解决了！您可以借助一些第三方库（如 `ouroboros`）来实现这一点。它提供了一种定义自引用结构的简单方法。也可以使用 unsafe Rust 来实现（实际上，`ouroboros` 本身内部就使用了 unsafe Rust……）

我们已经利用 [`ouroboros`](https://docs.rs/ouroboros/latest/ouroboros/attr.self_referencing.html) 为您定义了自引用的 `MemtableIterator` 字段。您需要基于这个提供的结构来实现 `MemtableIterator` 逻辑和 `Memtable::scan` API。

## 任务 2：合并迭代器 (Merge Iterator)

在此任务中，您需要修改：

```
src/iterators/merge_iterator.rs
```

既然您有多个内存表，并且将创建多个内存表迭代器。您需要合并来自内存表的结果，并将每个键的最新版本返回给用户。

`MergeIterator` 内部维护一个二叉堆 (binary heap)。考虑将 `n` 个已排序序列（我们的迭代器）合并为单个已排序输出的挑战；二叉堆在这里非常适合，因为它有效地帮助识别哪个序列当前拥有全局最小的元素。您会看到二叉堆的排序方式是，头部键值最小的迭代器排在最前面。当多个迭代器具有相同的头部键值时，最新的迭代器排在最前面。请注意，您需要处理错误（即，当迭代器无效时），并确保键值对的最新版本能够输出。

例如，如果我们有以下数据：

```
iter1: b->del, c->4, d->5
iter2: a->1, b->2, c->3
iter3: e->4
```

合并迭代器输出的序列应该是：

```
a->1, b->del, c->4, d->5, e->4
```

合并迭代器的构造函数接受一个迭代器向量。我们假设索引较低的迭代器（即第一个）拥有最新的数据。

使用 Rust 二叉堆时，您可能会发现 `peek_mut` 函数很有用。

```rust,no_run
let Some(mut inner) = heap.peek_mut() {
    *inner += 1; // <- 对内部项进行一些修改
}
// 当 PeekMut 引用被销毁时，二叉堆会自动重新排序。

let Some(mut inner) = heap.peek_mut() {
    PeekMut::pop(inner) // <- 将其从堆中弹出
}
```

一个常见的陷阱是错误处理。例如，

```rust,no_run
let Some(mut inner_iter) = self.iters.peek_mut() {
    inner_iter.next()?; // <- 会导致问题
}
```

如果 `next` 返回错误（例如，由于磁盘故障、网络故障、校验和错误等），则它不再有效。然而，当我们跳出 if 条件并将错误返回给调用者时，`PeekMut` 的 drop 会尝试在堆中移动元素，这会导致访问无效的迭代器。因此，您需要自己处理所有错误，而不是在 `PeekMut` 的作用域内使用 `?`。

我们希望尽可能避免动态分派 (dynamic dispatch)，因此我们不在系统中使用 `Box<dyn StorageIterator>`。相反，我们更喜欢使用泛型进行静态分派。另请注意，`StorageIterator` 使用泛型关联类型 (GAT)，因此它可以支持 `KeySlice` 和 `&[u8]` 作为键类型。我们将在第 3 周将时间戳添加到 `KeySlice` 中，现在为其使用单独的类型可以使过渡更加平滑。

从本节开始，我们将使用 `Key<T>` 来表示 LSM 键类型，并在类型系统中将它们与值区分开来。您应该使用 `Key<T>` 提供的 API，而不是直接访问内部值。我们将在第 3 部分将时间戳添加到此键类型中，使用键抽象将使过渡更加平滑。目前，`KeySlice` 等效于 `&[u8]`，`KeyVec` 等效于 `Vec<u8>`，`KeyBytes` 等效于 `Bytes`。

## 任务 3：LSM 迭代器 + Fused 迭代器 (LSM Iterator + Fused Iterator)

在此任务中，您需要修改：

```
src/lsm_iterator.rs
```

我们使用 `LsmIterator` 结构来表示内部 LSM 迭代器。在整个课程中，当向系统中添加更多迭代器时，您需要多次修改此结构。目前，因为我们只有多个内存表，所以它应该定义为：

```rust,no_run
type LsmIteratorInner = MergeIterator<MemTableIterator>;
```

您可以继续实现 `LsmIterator` 结构，该结构调用相应的内部迭代器，并跳过已删除的键。

我们在此任务中不测试 `LsmIterator`。任务 4 中将进行集成测试。

然后，我们希望为迭代器提供额外的安全性，以避免用户误用它们。当迭代器无效时，用户不应调用 `key`、`value` 或 `next`。同时，如果 `next` 返回错误，他们也不应再使用该迭代器。`FusedIterator` 是围绕迭代器的包装器，用于规范化所有迭代器的行为。您可以自行实现它。

## 任务 4：读取路径 - Scan (Read Path - Scan)

在此任务中，您需要修改：

```
src/lsm_storage.rs
```

我们终于完成了 —— 有了您实现的所有迭代器，您终于可以实现 LSM 引擎的 `scan` 接口了。您可以简单地使用内存表迭代器构造一个 LSM 迭代器（记住将最新的内存表放在合并迭代器的最前面），您的存储引擎将能够处理扫描请求。

## 测试您的理解 (Test Your Understanding)

* 使用您的合并迭代器的时间/空间复杂度是多少？
* 为什么我们需要为内存表迭代器使用自引用结构？
* 如果一个键被移除了（有一个删除墓碑），您需要将其返回给用户吗？您在哪里处理了这个逻辑？
* 如果一个键有多个版本，用户会看到所有版本吗？您在哪里处理了这个逻辑？
* 如果我们想摆脱自引用结构并在内存表迭代器上设置生命周期（例如，`MemtableIterator<'a>`，其中 `'a` = memtable 或 `LsmStorageInner` 的生命周期），是否仍然可以实现 `scan` 功能？
* 如果 (1) 我们在 skiplist 内存表上创建一个迭代器 (2) 有人向内存表中插入新键 (3) 迭代器会看到新键吗？
* 如果您的键比较器无法为二叉堆实现提供稳定的顺序会怎样？
* 为什么我们需要确保合并迭代器按迭代器构造顺序返回数据？
* 是否可以为 LSM 迭代器实现 Rust 风格的迭代器（即 `next(&self) -> (Key, Value)`）？优缺点是什么？
* scan 接口类似于 `fn scan(&self, lower: Bound<&[u8]>, upper: Bound<&[u8]>)`。如何使此 API 与 Rust 风格的范围（即 `key_a..key_b`）兼容？如果您实现了这一点，请尝试将完整范围 `..` 传递给该接口，看看会发生什么。
* 入门代码提供的合并迭代器接口存储 `Box<I>` 而不是 `I`。这背后的原因可能是什么？

我们不提供这些问题的参考答案，欢迎在 Discord 社区中讨论它们。

## 奖励任务 (Bonus Tasks)

* **前台迭代器 (Foreground Iterator)。** 在本课程中，我们假设所有操作都是短暂的，因此我们可以在迭代器中持有对内存表的引用。如果用户长时间持有迭代器，整个内存表（可能为 256MB）即使已刷写到磁盘，仍会保留在内存中。为了解决这个问题，我们可以向用户提供一个 `ForegroundIterator` / `LongIterator`。该迭代器将定期创建新的底层存储迭代器，以允许垃圾回收资源。

{{#include copyright.md}}
