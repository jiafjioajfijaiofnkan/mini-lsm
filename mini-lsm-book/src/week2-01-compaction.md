<!--
  mini-lsm-book © 2022-2025 by Alex Chi Z is licensed under CC BY-NC-SA 4.0
-->

# 压缩实现 (Compaction Implementation)

![本章概览](./lsm-tutorial/week2-01-full.svg)

在本章中，您将：

* 实现合并一些文件并生成新文件的压缩逻辑。
* 实现更新 LSM 状态和管理文件系统上 SST 文件的逻辑。
* 更新 LSM 读取路径以包含 LSM 层级。

要将测试用例复制到入门代码并运行它们：

```
cargo x copy-test --week 2 --day 1
cargo x scheck
```

<div class="warning">

在阅读本章之前，建议先查看 [第 2 周概览](./week2-overview.md) 以对压缩有一个总体了解。

</div>

## 任务 1：压缩实现 (Compaction Implementation)

在此任务中，您将实现执行压缩的核心逻辑 —— 将一组 SST 文件合并排序为一个有序的运行 (sorted run)。您需要修改：

```
src/compact.rs
```

具体来说，是 `force_full_compaction` 和 `compact` 函数。`force_full_compaction` 是决定哪些文件要压缩并更新 LSM 状态的压缩触发器。`compact` 执行实际的压缩作业，合并一些 SST 文件并返回一组新的 SST 文件。

您的压缩实现应该获取存储引擎中的所有 SST，使用 `MergeIterator` 对它们进行合并，然后使用 SST 构建器将结果写入新文件。如果文件太大，您需要拆分 SST 文件。压缩完成后，您可以更新 LSM 状态以将所有新的有序运行添加到 LSM 树的第一层。并且，您需要移除 LSM 树中未使用的文件。在您的实现中，SST 只应存储在两个地方：L0 SST 和 L1 SST。也就是说，LSM 状态中的 `levels` 结构应该只有一个向量。在 `LsmStorageState` 中，我们已经在 `levels` 字段中初始化了 LSM 以包含 L1。

压缩不应阻塞 L0 刷写，因此在合并文件时不应获取状态锁。您只应在压缩过程结束更新 LSM 状态时获取状态锁，并在完成状态修改后立即释放锁。

您可以假设用户将确保只有一个压缩正在进行。`force_full_compaction` 在任何时候都只会在一个线程中调用。放入第 1 层的 SST 应按其第一个键排序，并且不应具有重叠的键范围。

<details>

<summary>剧透：压缩伪代码</summary>

```rust,no_run
fn force_full_compaction(&self) {
    let ssts_to_compact = {
        let state = self.state.read();
        state.l0_sstables + state.levels[0]
    };
    let new_ssts = self.compact(FullCompactionTask(ssts_to_compact))?;
    {
        let state_lock = self.state_lock.lock();
        let state = self.state.write();
        state.l0_sstables.remove(/* 正在压缩的那些 */);
        state.levels[0] = new_ssts; // 新的 SST 添加到 L1
    };
    std::fs::remove(ssts_to_compact)?;
}
```

</details>

在您的压缩实现中，目前只需要处理 `FullCompaction`，其中任务信息包含您需要压缩的 SST。您还需要确保 SST 的顺序正确，以便键的最新版本将放入新的 SST 中。

因为我们总是压缩所有 SST，所以如果我们发现一个键有多个版本，我们可以简单地保留最新的版本。如果最新版本是删除标记 (delete marker)，则我们不需要将其保留在生成的 SST 文件中。这不适用于接下来几章中的压缩策略。

您可能需要考虑一些事情。

* 您的实现如何处理与压缩并行进行的 L0 刷写？（在进行压缩时不获取状态锁，并且还需要考虑压缩过程中产生的新 L0 文件。）
* 如果您的实现在压缩完成后立即删除原始 SST 文件，这是否会导致系统出现问题？（通常在 macOS/Linux 上不会，因为操作系统在没有文件句柄被持有时不会实际删除文件。）

## 任务 2：连接迭代器 (Concat Iterator)

在此任务中，您需要修改：

```
src/iterators/concat_iterator.rs
```

既然您已经在系统中创建了有序运行 (sorted run)，就可以对读取路径进行一个简单的优化。您不必总是为 SST 创建合并迭代器。如果 SST 属于一个有序运行，您可以创建一个连接迭代器 (concat iterator)，它只是按顺序迭代每个 SST 中的键，因为一个有序运行中的 SST 不包含重叠的键范围，并且它们按其第一个键排序。我们不希望预先创建所有 SST 迭代器（因为这会导致一次块读取），因此我们只在此迭代器中存储 SST 对象。

## 任务 3：与读取路径集成 (Integrate with the Read Path)

在此任务中，您需要修改：

```
src/lsm_iterator.rs
src/lsm_storage.rs
src/compact.rs
```

既然我们为 LSM 树设计了二级结构，您就可以更改读取路径以使用新的连接迭代器来优化读取路径。

您需要更改 `LsmStorageIterator` 的内部迭代器类型。之后，您可以构造一个双路合并迭代器来合并内存表和 L0 SST，以及另一个合并迭代器来将该迭代器与 L1 连接迭代器合并。

您还可以更改压缩实现以利用连接迭代器。

您需要为连接迭代器实现 `num_active_iterators`，以便测试用例可以测试您的实现是否正在使用连接迭代器，并且它应始终为 1。

要交互式地测试您的实现：

```shell
cargo run --bin mini-lsm-cli-ref -- --compaction none # 参考解决方案
cargo run --bin mini-lsm-cli -- --compaction none # 您的解决方案
```

然后，

```
fill 1000 3000
flush
fill 1000 3000
flush
full_compaction
fill 1000 3000
flush
full_compaction
get 2333
scan 2000 2333
```

## 测试您的理解 (Test Your Understanding)

* 读/写/空间放大的定义是什么？（这在概述章节中有所涉及）
* 准确计算读/写/空间放大的方法有哪些，估计它们的方法又有哪些？
* 即使用户请求删除一个键，该键仍会占用一些存储空间，这种说法是否正确？
* 鉴于压缩会占用大量写入带宽和读取带宽，并可能干扰前台操作，因此在写入流量较大时推迟压缩是一个好主意。在这种情况下，停止/暂停现有的压缩任务甚至是有益的。您对此有何看法？（阅读 [SILK: Preventing Latency Spikes in Log-Structured Merge Key-Value Stores](https://www.usenix.org/conference/atc19/presentation/balmau) 论文！）
* 为压缩使用/填充块缓存是个好主意吗？还是在压缩时完全绕过块缓存更好？
* 在系统中使用 `struct ConcatIterator<I: StorageIterator>` 是否有意义？
* 一些研究人员/工程师建议将压缩卸载到远程服务器或无服务器 lambda 函数。这样做的好处是什么，潜在的挑战和性能影响可能是什么？（思考一下压缩完成时的情况以及下一次读取请求时块缓存会发生什么……）

我们不提供这些问题的参考答案，欢迎在 Discord 社区中讨论它们。

{{#include copyright.md}}
