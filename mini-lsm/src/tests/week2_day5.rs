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

use std::time::Duration;

use bytes::BufMut;
use tempfile::tempdir;

use crate::{
    compact::{
        CompactionOptions, LeveledCompactionOptions, SimpleLeveledCompactionOptions,
        TieredCompactionOptions,
    },
    lsm_storage::{LsmStorageOptions, MiniLsm},
    tests::harness::dump_files_in_dir,
};

#[test]
fn test_integration_leveled() {
    test_integration(CompactionOptions::Leveled(LeveledCompactionOptions {
        level_size_multiplier: 2,
        level0_file_num_compaction_trigger: 2,
        max_levels: 3,
        base_level_size_mb: 1,
    }))
}

#[test]
fn test_integration_tiered() {
    test_integration(CompactionOptions::Tiered(TieredCompactionOptions {
        num_tiers: 3,
        max_size_amplification_percent: 200,
        size_ratio: 1,
        min_merge_width: 3,
        max_merge_width: None,
    }))
}

#[test]
fn test_integration_simple() {
    test_integration(CompactionOptions::Simple(SimpleLeveledCompactionOptions {
        size_ratio_percent: 200,
        level0_file_num_compaction_trigger: 2,
        max_levels: 3,
    }));
}

/// 配置存储，使 base_level 包含 2 个 SST 文件（目标大小为 2MB，每个 SST 为 1MB）。
/// 此配置的效果是压缩将生成一个新的较低级别，其中包含多个 SST 文件，
/// 分层压缩应正确处理这种情况：这些文件可能未按第一个键排序，
/// 并且在 manifest 恢复期间调用此函数时，不应在 `apply_compaction_result` 函数内部对其进行排序，
/// 因为此时我们没有加载任何实际的 SST。
#[test]
fn test_multiple_compacted_ssts_leveled() {
    let compaction_options = CompactionOptions::Leveled(LeveledCompactionOptions {
        level_size_multiplier: 4,
        level0_file_num_compaction_trigger: 2,
        max_levels: 2,
        base_level_size_mb: 2,
    });

    let lsm_storage_options = LsmStorageOptions::default_for_week2_test(compaction_options.clone());

    let dir = tempdir().unwrap();
    let storage = MiniLsm::open(&dir, lsm_storage_options).unwrap();

    // 插入大约 10MB 的数据，以确保至少有一次压缩是由优先级触发的。
    // 插入 500 个键值对，其中每个键值对为 2KB
    for i in 0..500 {
        let (key, val) = key_value_pair_with_target_size(i, 20 * 1024);
        storage.put(&key, &val).unwrap();
    }

    let mut prev_snapshot = storage.inner.state.read().clone();
    while {
        std::thread::sleep(Duration::from_secs(1));
        let snapshot = storage.inner.state.read().clone();
        let to_cont = prev_snapshot.levels != snapshot.levels
            || prev_snapshot.l0_sstables != snapshot.l0_sstables;
        prev_snapshot = snapshot;
        to_cont
    } {
        println!("等待压缩收敛");
    }

    storage.close().unwrap();
    assert!(storage.inner.state.read().memtable.is_empty());
    assert!(storage.inner.state.read().imm_memtables.is_empty());

    storage.dump_structure();
    drop(storage);
    dump_files_in_dir(&dir);

    let storage = MiniLsm::open(
        &dir,
        LsmStorageOptions::default_for_week2_test(compaction_options.clone()),
    )
    .unwrap();

    for i in 0..500 {
        let (key, val) = key_value_pair_with_target_size(i, 20 * 1024);
        assert_eq!(&storage.get(&key).unwrap().unwrap()[..], &val);
    }
}

fn test_integration(compaction_options: CompactionOptions) {
    let dir = tempdir().unwrap();
    let storage = MiniLsm::open(
        &dir,
        LsmStorageOptions::default_for_week2_test(compaction_options.clone()),
    )
    .unwrap();
    for i in 0..=20 {
        storage.put(b"0", format!("v{}", i).as_bytes()).unwrap();
        if i % 2 == 0 {
            storage.put(b"1", format!("v{}", i).as_bytes()).unwrap();
        } else {
            storage.delete(b"1").unwrap();
        }
        if i % 2 == 1 {
            storage.put(b"2", format!("v{}", i).as_bytes()).unwrap();
        } else {
            storage.delete(b"2").unwrap();
        }
        storage
            .inner
            .force_freeze_memtable(&storage.inner.state_lock.lock())
            .unwrap();
    }
    storage.close().unwrap();
    // 确保所有 SST 都已刷写
    assert!(storage.inner.state.read().memtable.is_empty());
    assert!(storage.inner.state.read().imm_memtables.is_empty());
    storage.dump_structure();
    drop(storage);
    dump_files_in_dir(&dir);

    let storage = MiniLsm::open(
        &dir,
        LsmStorageOptions::default_for_week2_test(compaction_options.clone()),
    )
    .unwrap();
    assert_eq!(&storage.get(b"0").unwrap().unwrap()[..], b"v20".as_slice());
    assert_eq!(&storage.get(b"1").unwrap().unwrap()[..], b"v20".as_slice());
    assert_eq!(storage.get(b"2").unwrap(), None);
}

/// 创建一个键和值都具有目标大小（字节）的键值对
fn key_value_pair_with_target_size(seed: i32, target_size_byte: usize) -> (Vec<u8>, Vec<u8>) {
    let mut key = vec![0; target_size_byte - 4];
    key.put_i32(seed);

    let mut val = vec![0; target_size_byte - 4];
    val.put_i32(seed);

    (key, val)
}
