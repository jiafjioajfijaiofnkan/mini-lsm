<!--
  mini-lsm-book © 2022-2025 by Alex Chi Z is licensed under CC BY-NC-SA 4.0
-->

# 前言 (Preface)

![Banner](./mini-lsm-logo.png)

本课程教您如何使用 Rust 构建一个简单的 LSM-Tree (日志结构合并树) 存储引擎。

## 什么是 LSM，为什么选择 LSM？ (What is LSM, and Why LSM?)

日志结构合并树是一种维护键值对的数据结构。这种数据结构广泛应用于分布式数据库系统，如 [TiDB](https://www.pingcap.com) 和 [CockroachDB](https://www.cockroachlabs.com)，作为其底层存储引擎。[RocksDB](http://rocksdb.org) 基于 [LevelDB](https://github.com/google/leveldb)，是 LSM-Tree 存储引擎的一种实现。它提供了许多键值访问功能，并被用于许多生产系统。

一般来说，LSM 树是一种对追加操作友好的数据结构。将其与 RB-Tree (红黑树) 和 B-Tree (B 树) 等其他键值数据结构进行比较会更直观。对于 RB-Tree 和 B-Tree，所有数据操作都是原地进行的。也就是说，当您想要更新与键对应的值时，引擎会用新值覆盖其原始内存或磁盘空间。但在 LSM 树中，所有写入操作，即插入、更新、删除，都是延迟应用于存储的。引擎将这些操作批量处理到 SST (Sorted String Table, 有序字符串表) 文件中，并将它们写入磁盘。一旦写入磁盘，引擎就不会直接修改它们。在一个称为压缩 (compaction) 的特定后台任务中，引擎将合并这些文件以应用更新和删除。

这种架构设计使得 LSM 树易于使用。

1. 数据在持久存储上是不可变的。并发控制更加直接。将后台任务（压缩）卸载到远程服务器是可能的。直接从 S3 等云原生存储系统存储和提供数据也是可行的。
2. 更改压缩算法允许存储引擎在读取、写入和空间放大之间进行平衡。数据结构用途广泛，通过调整压缩参数，我们可以针对不同的工作负载优化 LSM 结构。

本课程将教您如何使用 Rust 编程语言构建一个基于 LSM 树的存储引擎。

## 先修条件 (Prerequisites)

* 您应该了解 Rust 编程语言的基础知识。阅读 [Rust 官方教程](https://doc.rust-lang.org/book/) 就足够了。
* 您应该了解键值存储引擎的基本概念，即为什么我们需要复杂的设计来实现持久性。如果您以前没有数据库系统和存储系统的经验，您可以在 [PingCAP Talent Plan](https://github.com/pingcap/talent-plan/tree/master/courses/rust/projects/project-2) 中实现 Bitcask。
* 了解 LSM 树的基础知识不是必需的，但我们建议您阅读一些相关资料，例如 LevelDB 的总体思想。提前了解这些概念将使您熟悉诸如可变和不可变 memtable、SST、压缩、WAL (预写日志) 等概念。

## 您能从本课程中期待什么 (What should you expect from this course)

完成本课程后，您应该对基于 LSM 的存储系统的工作原理有深入的了解，获得设计此类系统的实践经验，并将所学知识应用于您的学习和职业生涯。您将理解此类存储系统中的设计权衡，并找到设计基于 LSM 的存储系统的最佳方法，以满足您的工作负载要求/目标。本课程非常深入，涵盖了现代存储系统（即 RocksDB）的所有基本实现细节和设计选择，基于作者在多个类似 LSM 的存储系统中的经验，您将能够直接将所学知识应用于工业界和学术界。

### 结构 (Structure)

本课程是一门内容广泛的课程，分为几个部分（周）。每周有七个章节；您可以在 2 到 3 小时内完成每个章节。每个部分的前六章将指导您构建一个可工作的系统，每周的最后一章将是 *点心时间* (snack time) 章节，在您前六天构建的基础上实现一些简单的东西。每个章节都将包含必需的任务、*检查您的理解* (check your understanding) 问题和奖励任务。

### 测试 (Testing)

我们提供了一个完整的测试套件和一些 CLI (命令行界面) 工具，供您验证您的解决方案是否正确。请注意，测试套件并非详尽无遗，您的解决方案在通过所有测试用例后可能并非 100% 正确。在实现系统的后续部分时，您可能需要修复早期版本的错误。我们建议您仔细考虑您的实现，尤其是在涉及多线程操作和竞争条件 (race condition) 时。

### 解决方案 (Solution)

我们在 mini-lsm 主仓库中提供了一个解决方案，实现了课程中要求的所有功能。同时，我们还有一个 mini-lsm 解决方案检查点仓库，其中每个提交都对应课程中的一个章节。

保持这样一个检查点仓库与 mini-lsm 课程同步是一项挑战，因为每个错误修复或新功能都必须经过所有提交（或检查点）。因此，此仓库可能未使用最新的入门代码或包含 mini-lsm 课程的最新功能。

**长话短说：我们不保证解决方案检查点仓库包含正确的解决方案、通过所有测试或具有正确的文档注释。** 如需正确的实现以及实现所有内容后的解决方案，请查看主仓库中的解决方案：[https://github.com/skyzh/mini-lsm/tree/main/mini-lsm](https://github.com/skyzh/mini-lsm/tree/main/mini-lsm)。

如果您在课程的某个部分遇到困难，或者需要帮助确定在何处实现功能，您可以参考此仓库以获取帮助。您可以比较提交之间的差异以了解更改的内容。您可能需要在整个章节中多次修改 mini-lsm 课程中的某些函数，您可以在此仓库中了解每个章节预期实现的具体内容。

您可以访问解决方案检查点仓库：[https://github.com/skyzh/mini-lsm-solution-checkpoint](https://github.com/skyzh/mini-lsm-solution-checkpoint)。

### 反馈 (Feedbacks)

非常感谢您的反馈。我们根据学生的反馈在 2024 年从头开始重写了整个课程。请分享您的学习经验，帮助我们不断改进课程。欢迎加入 [Discord 社区](https://skyzh.dev/join/discord) 分享您的经验。

我们重写课程的漫长故事：该课程最初计划为一个通用指南，学生从一个空目录开始，根据我们提供的规范实现他们想要的任何内容。我们只有最少的测试来检查行为是否正确。然而，原始课程过于开放，给学习体验带来了巨大障碍。由于学生事先对整个系统没有概览，而且说明含糊不清，有时他们很难知道为什么做出某个设计决策以及他们需要什么来实现目标。课程的某些部分内容过于紧凑，无法在一章内完成预期的内容。因此，我们彻底重新设计了课程，以提供更轻松的学习曲线和更清晰的学习目标。原来为期一周的课程现在分为两周（第一周关于存储格式，第二周深入研究压缩），外加一个关于 MVCC (多版本并发控制) 的额外部分。我们希望您觉得本课程有趣，并对您的学习和职业生涯有所帮助。我们要感谢所有在 [Feedback after coding day 1](https://github.com/skyzh/mini-lsm/issues/11) 和 [Hello, when is the next update plan for the course?](https://github.com/skyzh/mini-lsm/issues/7) 中发表评论的人 —— 您的反馈极大地帮助我们改进了课程。

### 许可证 (License)

本课程的源代码采用 Apache 2.0 许可证授权，而本书采用 CC BY-NC-SA 4.0 许可证授权。

### 本课程会永远免费吗？ (Will this course be free forever?)

是的！现在所有公开可用的内容都将永远免费，并将获得终身更新和错误修复。同时，我们可能会提供付费的代码审查和答疑服务。对于 DLC (可下载内容) 部分（*你的余生* (rest of your life) 章节），截至 2024 年我们还没有完成它们的计划，也尚未决定它们是否会公开可用。

## 社区 (Community)

您可以加入 skyzh 的 Discord 服务器与 mini-lsm 社区一起学习。

[![加入 skyzh 的 Discord 服务器](discord-badge.svg)](https://skyzh.dev/join/discord)

## 开始学习 (Get Started)

现在，您可以在 [Mini-LSM 课程概览](./00-overview.md) 中了解 LSM 结构的概览。

## 关于作者 (About the Author)

在撰写本文时（2024 年初），Chi 获得了卡内基梅隆大学的计算机科学硕士学位和上海交通大学的学士学位。他曾参与多个数据库系统的开发，包括 [TiKV][db1]、[AgateDB][db2]、[TerarkDB][db3]、[RisingWave][db4] 和 [Neon][db5]。自 2022 年以来，他担任了三个学期的 [CMU 数据库系统课程](https://15445.courses.cs.cmu) 的助教，负责 BusTub 教学系统，期间为该课程添加了许多新功能和更具挑战性的内容（请查看重新设计的[查询执行](https://15445.courses.cs.cmu.edu/fall2022/project3/)项目和极具挑战性的[多版本并发控制](https://15445.courses.cs.cmu.edu/fall2023/project4/)项目）。除了参与 BusTub 教学系统的开发，他还维护着 [RisingLight](https://github.com/risinglightdb/risinglight) 教学数据库系统。Chi 对探索 Rust 编程语言如何融入数据库领域非常感兴趣。如果您也对此主题感兴趣，可以查看他之前的关于构建向量化表达式框架的课程 [type-exercise-in-rust](https://github.com/skyzh/type-exercise-in-rust) 和关于构建向量数据库的课程 [write-you-a-vector-db](https://github.com/skyzh/write-you-a-vector-db)。

[db1]: https://github.com/tikv/tikv
[db2]: https://github.com/tikv/agatedb
[db3]: https://github.com/bytedance/terarkdb
[db4]: https://github.com/risingwavelabs/risingwave
[db5]: https://github.com/neondatabase/neon

{{#include copyright.md}}
