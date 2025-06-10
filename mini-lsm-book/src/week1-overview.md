<!--
  mini-lsm-book © 2022-2025 by Alex Chi Z is licensed under CC BY-NC-SA 4.0
-->

# 第 1 周概览：Mini-LSM (Week 1 Overview: Mini-LSM)

![本章概览](./lsm-tutorial/week1-overview.svg)

在本课程的第一周，您将为存储引擎构建必要的存储格式、系统的读取路径和写入路径，并拥有一个可工作的基于 LSM 的键值存储实现。这部分共有 7 个章节（天）。

* [第 1 天：内存表 (Memtable)](./week1-01-memtable.md)。您将实现系统的内存读取和写入路径。
* [第 2 天：合并迭代器 (Merge Iterator)](./week1-02-merge-iterator.md)。您将扩展第 1 天构建的内容，并为您的系统实现 `scan` 接口。
* [第 3 天：块编码 (Block Encoding)](./week1-03-block.md)。现在我们开始磁盘结构的第一步，构建块的编码/解码。
* [第 4 天：SST 编码 (SST Encoding)](./week1-04-sst.md)。SST 由块组成，在这一天结束时，您将拥有 LSM 磁盘结构的基本构建块。
* [第 5 天：读取路径 (Read Path)](./week1-05-read-path.md)。既然我们同时拥有内存和磁盘结构，我们可以将它们组合在一起，为存储引擎提供一个功能齐全的读取路径。
* [第 6 天：写入路径 (Write Path)](./week1-06-write-path.md)。在第 5 天，测试工具会生成结构，而在第 6 天，您将自己控制 SST 的刷写。您将实现刷写到 0 级 (level-0) SST，存储引擎就完成了。
* [第 7 天：SST 优化 (SST Optimizations)](./week1-07-sst-optimizations.md)。我们将实现几种 SST 格式优化并提高系统性能。

在本周末，您的存储引擎应该能够处理所有 get/scan/put 请求。唯一缺少的部分是将 LSM 状态持久化到磁盘，以及一种更有效地在磁盘上组织 SST 的方法。您将拥有一个可工作的 **Mini-LSM** 存储引擎。

{{#include copyright.md}}
