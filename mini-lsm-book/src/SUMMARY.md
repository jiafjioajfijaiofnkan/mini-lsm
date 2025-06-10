<!--
  mini-lsm-book © 2022-2025 by Alex Chi Z is licensed under CC BY-NC-SA 4.0
-->

# LSM in a Week (一周实现 LSM)

[前言 (Preface)](./00-preface.md)
[Mini-LSM 概览 (Mini-LSM Overview)](./00-overview.md)
[环境配置 (Environment Setup)](./00-get-started.md)

- [第 1 周概览：Mini-LSM (Week 1 Overview: Mini-LSM)](./week1-overview.md)
  - [内存表 (Memtable)](./week1-01-memtable.md)
  - [合并迭代器 (Merge Iterator)](./week1-02-merge-iterator.md)
  - [块 (Block)](./week1-03-block.md)
  - [有序字符串表 (Sorted String Table, SST)](./week1-04-sst.md)
  - [读取路径 (Read Path)](./week1-05-read-path.md)
  - [写入路径 (Write Path)](./week1-06-write-path.md)
  - [点心时间：SST 优化 (Snack Time: SST Optimizations)](./week1-07-sst-optimizations.md)

- [第 2 周概览：压缩 + 持久化 (Week 2 Overview: Compaction + Persistence)](./week2-overview.md)
  - [压缩实现 (Compaction Implementation)](./week2-01-compaction.md)
  - [简单压缩策略 (Simple Compaction Strategy)](./week2-02-simple.md)
  - [分层压缩策略 (Tiered Compaction Strategy)](./week2-03-tiered.md)
  - [级别压缩策略 (Leveled Compaction Strategy)](./week2-04-leveled.md)
  - [Manifest (清单)](./week2-05-manifest.md)
  - [预写日志 (Write-Ahead Log, WAL)](./week2-06-wal.md)
  - [点心时间：批量写入和校验和 (Snack Time: Batch Write and Checksums)](./week2-07-snacks.md)

- [第 3 周概览：MVCC (多版本并发控制) (Week 3 Overview: MVCC)](./week3-overview.md)
  - [时间戳编码 + 重构 (Timestamp Encoding + Refactor)](./week3-01-ts-key-refactor.md)
  - [快照 - 内存表和时间戳 (Snapshots - Memtables and Timestamps)](./week3-02-snapshot-read-part-1.md)
  - [快照 - 事务 API (Snapshots - Transaction API)](./week3-03-snapshot-read-part-2.md)
  - [水印和垃圾回收 (Watermark and GC)](./week3-04-watermark.md)
  - [事务和乐观并发控制 (Transaction and OCC)](./week3-05-txn-occ.md)
  - [可串行化快照隔离 (Serializable Snapshot Isolation)](./week3-06-serializable.md)
  - [点心时间：压缩过滤器 (Snack Time: Compaction Filters)](./week3-07-compaction-filter.md)
- [你的余生 (待定) (The Rest of Your Life (TBD))](./week4-overview.md)

---

# 已弃用 Mini-LSM v1 (DEPRECATED Mini-LSM v1)

- [概览 (Overview)](./00-v1.md)
  - [将键值对存储在小块中 (Store key-value pairs in little blocks)](./01-block.md)
  - [并将它们制作成 SST (And make them into an SST)](./02-sst.md)
  - [现在是时候合并所有内容了 (Now it's time to merge everything)](./03-memtable.md)
  - [引擎着火了 (The engine is on fire)](./04-engine.md)
  - [让我们在后台做些事情 (Let's do something in the background)](./05-compaction.md)
  - [系统崩溃时要小心 (Be careful when the system crashes)](./06-recovery.md)
  - [一个好的布隆过滤器让生活更轻松 (A good bloom filter makes life easier)](./07-bloom-filter.md)
  - [希望能节省一些空间 (Save some space, hopefully)](./08-key-compression.md)
  - [接下来是什么 (What's next)](./09-whats-next.md)
