// 版权所有 (c) 2022-2025 Alex Chi Z
//
// 本软件根据 Apache 许可证 2.0 版本（以下简称“许可证”）获得许可；
// 除非遵守许可证，否则您不得使用本文件。
// 您可以在以下网址获取许可证副本：
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// 除非适用法律要求或书面同意，根据许可证分发的软件
// 均以“原样”提供，不附带任何明示或暗示的保证或条件。
// 请参阅许可证以了解特定语言下的权限和限制。

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::lsm_storage::LsmStorageState;

#[derive(Debug, Clone)]
pub struct SimpleLeveledCompactionOptions {
    pub size_ratio_percent: usize,
    pub level0_file_num_compaction_trigger: usize,
    pub max_levels: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SimpleLeveledCompactionTask {
    // 如果 upper_level 为 `None`，则表示是 L0 压缩
    pub upper_level: Option<usize>,
    pub upper_level_sst_ids: Vec<usize>,
    pub lower_level: usize,
    pub lower_level_sst_ids: Vec<usize>,
    pub is_lower_level_bottom_level: bool,
}

pub struct SimpleLeveledCompactionController {
    options: SimpleLeveledCompactionOptions,
}

impl SimpleLeveledCompactionController {
    pub fn new(options: SimpleLeveledCompactionOptions) -> Self {
        Self { options }
    }

    /// 生成一个压缩任务。
    ///
    /// 如果不需要调度压缩，则返回 `None`。压缩任务 ID 向量中 SST 的顺序很重要。
    pub fn generate_compaction_task(
        &self,
        snapshot: &LsmStorageState,
    ) -> Option<SimpleLeveledCompactionTask> {
        if self.options.max_levels == 0 {
            return None;
        }

        let mut level_sizes = Vec::new();
        level_sizes.push(snapshot.l0_sstables.len());
        for (_, files) in &snapshot.levels {
            level_sizes.push(files.len());
        }

        // 检查 L0 到 L1 压缩的 level0_file_num_compaction_trigger
        if snapshot.l0_sstables.len() >= self.options.level0_file_num_compaction_trigger {
            println!(
                "在层级 0 触发压缩，因为 L0 有 {} 个 SST >= {}",
                snapshot.l0_sstables.len(),
                self.options.level0_file_num_compaction_trigger
            );
            return Some(SimpleLeveledCompactionTask {
                upper_level: None,
                upper_level_sst_ids: snapshot.l0_sstables.clone(),
                lower_level: 1,
                lower_level_sst_ids: snapshot.levels[0].1.clone(),
                is_lower_level_bottom_level: false,
            });
        }

        // 检查其他层级 (>= L1) 压缩的 size_ratio_percent
        for i in 1..self.options.max_levels {
            let lower_level = i + 1;
            let size_ratio = level_sizes[lower_level] as f64 / level_sizes[i] as f64;
            if size_ratio < self.options.size_ratio_percent as f64 / 100.0 {
                println!(
                    "在层级 {} 和 {} 触发压缩，大小比率为 {}",
                    i, lower_level, size_ratio
                );
                return Some(SimpleLeveledCompactionTask {
                    upper_level: Some(i),
                    upper_level_sst_ids: snapshot.levels[i - 1].1.clone(),
                    lower_level,
                    lower_level_sst_ids: snapshot.levels[lower_level - 1].1.clone(),
                    is_lower_level_bottom_level: lower_level == self.options.max_levels,
                });
            }
        }
        None
    }

    /// 应用压缩结果。
    ///
    /// 压缩器将使用压缩任务和生成的 SST ID 列表调用此函数。此函数应用
    /// 结果并生成新的 LSM 状态。这些函数只应更改 `l0_sstables` 和 `levels`，
    /// 而不更改 memtables 和 `sstables` 哈希映射。虽然应该只有一个线程运行压缩作业，
    /// 但您应该考虑在压缩器生成新的 SST 时 L0 SST 被刷写的情况，
    /// 考虑到这一点，您应该在实现中进行一些健全性检查。
    pub fn apply_compaction_result(
        &self,
        snapshot: &LsmStorageState,
        task: &SimpleLeveledCompactionTask,
        output: &[usize],
    ) -> (LsmStorageState, Vec<usize>) {
        let mut snapshot = snapshot.clone();
        let mut files_to_remove = Vec::new();
        if let Some(upper_level) = task.upper_level {
            assert_eq!(
                task.upper_level_sst_ids,
                snapshot.levels[upper_level - 1].1,
                "SST 不匹配"
            );
            files_to_remove.extend(&snapshot.levels[upper_level - 1].1);
            snapshot.levels[upper_level - 1].1.clear();
        } else {
            files_to_remove.extend(&task.upper_level_sst_ids);
            let mut l0_ssts_compacted = task
                .upper_level_sst_ids
                .iter()
                .copied()
                .collect::<HashSet<_>>();
            let new_l0_sstables = snapshot
                .l0_sstables
                .iter()
                .copied()
                .filter(|x| !l0_ssts_compacted.remove(x))
                .collect::<Vec<_>>();
            assert!(l0_ssts_compacted.is_empty());
            snapshot.l0_sstables = new_l0_sstables;
        }
        assert_eq!(
            task.lower_level_sst_ids,
            snapshot.levels[task.lower_level - 1].1,
            "SST 不匹配"
        );
        files_to_remove.extend(&snapshot.levels[task.lower_level - 1].1);
        snapshot.levels[task.lower_level - 1].1 = output.to_vec();
        (snapshot, files_to_remove)
    }
}
