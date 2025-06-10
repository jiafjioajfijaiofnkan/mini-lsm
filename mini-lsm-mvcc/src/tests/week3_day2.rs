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

use tempfile::tempdir;

use crate::{
    compact::CompactionOptions,
    lsm_storage::{LsmStorageOptions, MiniLsm},
    tests::harness::dump_files_in_dir,
};

#[test]
fn test_task3_compaction_integration() {
    let dir = tempdir().unwrap();
    let mut options = LsmStorageOptions::default_for_week2_test(CompactionOptions::NoCompaction);
    options.enable_wal = true;
    let storage = MiniLsm::open(&dir, options.clone()).unwrap();
    let _txn = storage.new_txn().unwrap();
    for i in 0..=20000 {
        storage
            .put(b"0", format!("{:02000}", i).as_bytes())
            .unwrap();
    }
    std::thread::sleep(Duration::from_secs(1)); // 等待所有内存表刷写
    while {
        let snapshot = storage.inner.state.read();
        !snapshot.imm_memtables.is_empty()
    } {
        storage.inner.force_flush_next_imm_memtable().unwrap();
    }
    assert!(storage.inner.state.read().l0_sstables.len() > 1);
    storage.force_full_compaction().unwrap();
    storage.dump_structure();
    dump_files_in_dir(&dir);
    assert!(storage.inner.state.read().l0_sstables.is_empty());
    assert_eq!(storage.inner.state.read().levels.len(), 1);
    // 同一个 SST 中的相同键
    assert_eq!(storage.inner.state.read().levels[0].1.len(), 1);
    for i in 0..=100 {
        storage
            .put(b"1", format!("{:02000}", i).as_bytes())
            .unwrap();
    }
    storage
        .inner
        .force_freeze_memtable(&storage.inner.state_lock.lock())
        .unwrap();
    std::thread::sleep(Duration::from_secs(1)); // 等待所有内存表刷写
    while {
        let snapshot = storage.inner.state.read();
        !snapshot.imm_memtables.is_empty()
    } {
        storage.inner.force_flush_next_imm_memtable().unwrap();
    }
    storage.force_full_compaction().unwrap();
    storage.dump_structure();
    dump_files_in_dir(&dir);
    assert!(storage.inner.state.read().l0_sstables.is_empty());
    assert_eq!(storage.inner.state.read().levels.len(), 1);
    // 同一个 SST 中的相同键，现在我们应该拆分为两个
    assert_eq!(storage.inner.state.read().levels[0].1.len(), 2);
}
