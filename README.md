![banner](./mini-lsm-book/src/mini-lsm-logo.png)

# 一周实现 LSM 树 (LSM in a Week)

[![CI (main)](https://github.com/skyzh/mini-lsm/actions/workflows/main.yml/badge.svg)](https://github.com/skyzh/mini-lsm/actions/workflows/main.yml)

一周内构建一个简单的键值 (key-value) 存储引擎！并在第二周和第三周扩展您的 LSM 引擎。

## [教程 (Book)](https://skyzh.github.io/mini-lsm)

Mini-LSM 教程可在 [https://skyzh.github.io/mini-lsm](https://skyzh.github.io/mini-lsm) 获取。您可以按照本指南实现 Mini-LSM 存储引擎。我们的课程分为三周（部分），每周包含 7 天（章节）的内容。

## 社区 (Community)

您可以加入 skyzh 的 Discord 服务器与 mini-lsm 社区一起学习。

[![加入 skyzh 的 Discord 服务器](mini-lsm-book/src/discord-badge.svg)](https://skyzh.dev/join/discord)

**添加您的解决方案 (Add Your Solution)**

如果您完成了本课程至少一周的全部内容，您可以将您的解决方案添加到 [SOLUTIONS.md](./SOLUTIONS.md) 中的社区解决方案列表中。您可以提交一个拉取请求 (pull request)，作为您辛勤工作的回报，我们可能会对您的代码进行快速审查。

## 开发 (Development)

**对于学生 (For Students)**

您应该修改 `mini-lsm-starter` 目录中的代码。

```
cargo x install-tools
cargo x copy-test --week 1 --day 1
cargo x scheck
cargo run --bin mini-lsm-cli
cargo run --bin compaction-simulator
```

**对于课程开发者 (For Course Developers)**

您应该修改 `mini-lsm` 和 `mini-lsm-mvcc`。

```
cargo x install-tools
cargo x check
cargo x book
```

如果您更改了参考解决方案中的公共 API (public API)，您可能还需要将其同步到入门项目 (starter crate) 中。
为此，请使用 `cargo x sync`。

## 代码结构 (Code Structure)

* mini-lsm: <= 第二周的最终解决方案代码
* mini-lsm-mvcc: 第三周 MVCC (多版本并发控制) 的最终解决方案代码
* mini-lsm-starter: 入门代码
* mini-lsm-book: 课程

我们还有一个名为 mini-lsm-solution-checkpoint 的仓库，地址在 [https://github.com/skyzh/mini-lsm-solution-checkpoint](https://github.com/skyzh/mini-lsm-solution-checkpoint)。在这个仓库中，每个提交 (commit) 都对应课程中的一个章节。我们不会非常频繁地更新解决方案检查点 (solution checkpoint)。

## 演示 (Demo)

在开始之前，您可以自行运行参考解决方案以了解系统概况。

```
cargo run --bin mini-lsm-cli-ref
cargo run --bin mini-lsm-cli-mvcc-ref
```

我们还有一个压缩模拟器 (compaction simulator) 来试验您的压缩算法 (compaction algorithm) 实现，

```
cargo run --bin compaction-simulator-ref
cargo run --bin compaction-simulator-mvcc-ref
```

## 课程结构 (Course Structure)

本课程包含 3 周 + 1 个额外周（进行中）的内容。

* 第 1 周：存储格式 (Storage Format) + 引擎骨架 (Engine Skeleton)
* 第 2 周：压缩 (Compaction) 和持久化 (Persistence)
* 第 3 周：多版本并发控制 (Multi-Version Concurrency Control)
* 额外一周/你的余生 (The Extra Week / Rest of Your Life): 优化 (Optimizations) (不太可能在 2025 年提供...)

![课程路线图 (Course Roadmap)](./mini-lsm-book/src/lsm-tutorial/00-full-overview.svg)

| 周 + 章节 (Week + Chapter) | 主题 (Topic)                                                       |
| -------------- | ----------------------------------------------------------- |
| 1.1            | Memtable (内存表)                                           |
| 1.2            | Merge Iterator (合并迭代器)                                 |
| 1.3            | Block (块)                                                  |
| 1.4            | Sorted String Table (SST) (有序字符串表)                    |
| 1.5            | Read Path (读取路径)                                        |
| 1.6            | Write Path (写入路径)                                       |
| 1.7            | SST 优化 (Optimizations): 前缀键编码 (Prefix Key Encoding) + 布隆过滤器 (Bloom Filters) |
| 2.1            | Compaction Implementation (压缩实现)                        |
| 2.2            | Simple Compaction Strategy (简单压缩策略) (Traditional Leveled Compaction) (传统分层压缩) |
| 2.3            | Tiered Compaction Strategy (分层压缩策略) (RocksDB Universal Compaction) (RocksDB 通用压缩) |
| 2.4            | Leveled Compaction Strategy (级别压缩策略) (RocksDB Leveled Compaction) (RocksDB 级别压缩) |
| 2.5            | Manifest                                                    |
| 2.6            | Write-Ahead Log (WAL) (预写日志)                            |
| 2.7            | Batch Write (批量写入) and Checksums (校验和)               |
| 3.1            | Timestamp Key Encoding (时间戳键编码)                       |
| 3.2            | Snapshot Read (快照读取) - Memtables and Timestamps (内存表和时间戳) |
| 3.3            | Snapshot Read (快照读取) - Transaction API (事务 API)       |
| 3.4            | Watermark (水印) and Garbage Collection (垃圾回收)          |
| 3.5            | Transactions (事务) and Optimistic Concurrency Control (乐观并发控制) |
| 3.6            | Serializable Snapshot Isolation (可串行化快照隔离)         |
| 3.7            | Compaction Filters (压缩过滤器)                             |

## 相关项目 (Related Projects)

mini-lsm 启发了几个用于生产的项目。

* [SlateDB](https://slatedb.io/docs/architecture/) 是一个基于对象存储 (object storage) 系统的 LSM 引擎。
* [Tonbo](https://tonbo.io/about) 直接在对象存储 (object storage) 上存储 parquet 文件，并以 LSM 树结构组织它们。

## 许可证 (License)

Mini-LSM 入门代码和解决方案采用 [Apache 2.0 许可证 (Apache 2.0 license)](LICENSE)。作者保留课程材料（markdown 文件和图表）的全部版权。
