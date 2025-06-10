<!--
  mini-lsm-book © 2022-2025 by Alex Chi Z is licensed under CC BY-NC-SA 4.0
-->

# Mini-LSM 课程概览 (Course Overview)

## 课程结构 (Course Structure)

![课程概览 (Course Overview)](lsm-tutorial/00-full-overview.svg)

本课程包含三个部分（周）。在第一周，我们将重点关注 LSM 存储引擎的存储结构和存储格式。在第二周，我们将深入研究压缩 (compaction) 并为存储引擎实现持久化支持。在第三周，我们将实现多版本并发控制 (multi-version concurrency control)。

* [第一周：Mini-LSM](./week1-overview.md)
* [第二周：压缩与持久化](./week2-overview.md)
* [第三周：多版本并发控制](./week3-overview.md)

请查看 [环境配置](./00-get-started.md) 来配置您的环境。

## LSM 概览 (Overview of LSM)

LSM 存储引擎通常包含三个部分：

1. 预写日志 (Write-ahead log)，用于持久化临时数据以供恢复。
2. 磁盘上的 SST (Sorted String Table)，用于维护 LSM 树结构。
3. 内存中的 MemTable (内存表)，用于批量处理小量写入。

存储引擎通常提供以下接口：

* `Put(key, value)`：在 LSM 树中存储一个键值对。
* `Delete(key)`：移除一个键及其对应的值。
* `Get(key)`：获取与键对应的值。
* `Scan(range)`：获取一个范围内的键值对。

为了确保持久性：

* `Sync()`：确保 `sync` 之前的所有操作都已持久化到磁盘。

一些引擎选择将 `Put` 和 `Delete` 合并为单个操作，称为 `WriteBatch`，它接受一批键值对。

在本课程中，我们假设 LSM 树使用分层压缩算法 (leveled compaction algorithm)，这在实际系统中很常用。

### 写入路径 (Write Path)

![写入路径 (Write Path)](lsm-tutorial/00-lsm-write-flow.svg)

LSM 的写入路径包含四个步骤：

1. 将键值对写入预写日志，以便在存储引擎崩溃后可以恢复。
2. 将键值对写入 memtable。完成 (1) 和 (2) 后，我们可以通知用户写入操作已完成。
3. (后台) 当 memtable 已满时，我们会将其冻结为不可变 memtable，并在后台将其作为 SST 文件刷写到磁盘。
4. (后台) 引擎会将某些层级中的一些文件压缩到更低的层级，以保持 LSM 树的良好形状，从而降低读取放大 (read amplification)。

### 读取路径 (Read Path)

![读取路径 (Read Path)](lsm-tutorial/00-lsm-read-flow.svg)

当我们想要读取一个键时：

1. 我们将首先从最新到最旧探测所有的 memtable。
2. 如果未找到该键，我们将在包含 SST 的整个 LSM 树中搜索数据。

有两种类型的读取：查找 (lookup) 和扫描 (scan)。查找是在 LSM 树中查找单个键，而扫描是在存储引擎中迭代指定范围内的所有键。我们将在整个课程中涵盖这两种操作。

{{#include copyright.md}}
