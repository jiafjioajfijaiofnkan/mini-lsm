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

use std::sync::Arc;

use tempfile::tempdir;

use crate::{
    lsm_storage::{LsmStorageInner, LsmStorageOptions},
    mem_table::MemTable,
};

#[test]
fn test_task1_memtable_get() {
    let memtable = MemTable::create(0);
    memtable.for_testing_put_slice(b"key1", b"value1").unwrap();
    memtable.for_testing_put_slice(b"key2", b"value2").unwrap();
    memtable.for_testing_put_slice(b"key3", b"value3").unwrap();
    assert_eq!(
        &memtable.for_testing_get_slice(b"key1").unwrap()[..],
        b"value1"
    );
    assert_eq!(
        &memtable.for_testing_get_slice(b"key2").unwrap()[..],
        b"value2"
    );
    assert_eq!(
        &memtable.for_testing_get_slice(b"key3").unwrap()[..],
        b"value3"
    );
}

#[test]
fn test_task1_memtable_overwrite() {
    let memtable = MemTable::create(0);
    memtable.for_testing_put_slice(b"key1", b"value1").unwrap();
    memtable.for_testing_put_slice(b"key2", b"value2").unwrap();
    memtable.for_testing_put_slice(b"key3", b"value3").unwrap();
    memtable.for_testing_put_slice(b"key1", b"value11").unwrap();
    memtable.for_testing_put_slice(b"key2", b"value22").unwrap();
    memtable.for_testing_put_slice(b"key3", b"value33").unwrap();
    assert_eq!(
        &memtable.for_testing_get_slice(b"key1").unwrap()[..],
        b"value11"
    );
    assert_eq!(
        &memtable.for_testing_get_slice(b"key2").unwrap()[..],
        b"value22"
    );
    assert_eq!(
        &memtable.for_testing_get_slice(b"key3").unwrap()[..],
        b"value33"
    );
}

#[test]
fn test_task2_storage_integration() {
    let dir = tempdir().unwrap();
    let storage = Arc::new(
        LsmStorageInner::open(dir.path(), LsmStorageOptions::default_for_week1_test()).unwrap(),
    );
    assert_eq!(&storage.get(b"0").unwrap(), &None);
    storage.put(b"1", b"233").unwrap();
    storage.put(b"2", b"2333").unwrap();
    storage.put(b"3", b"23333").unwrap();
    assert_eq!(&storage.get(b"1").unwrap().unwrap()[..], b"233");
    assert_eq!(&storage.get(b"2").unwrap().unwrap()[..], b"2333");
    assert_eq!(&storage.get(b"3").unwrap().unwrap()[..], b"23333");
    storage.delete(b"2").unwrap();
    assert!(storage.get(b"2").unwrap().is_none());
    storage.delete(b"0").unwrap(); // 不应报告任何错误
}

#[test]
fn test_task3_storage_integration() {
    let dir = tempdir().unwrap();
    let storage = Arc::new(
        LsmStorageInner::open(dir.path(), LsmStorageOptions::default_for_week1_test()).unwrap(),
    );
    storage.put(b"1", b"233").unwrap();
    storage.put(b"2", b"2333").unwrap();
    storage.put(b"3", b"23333").unwrap();
    storage
        .force_freeze_memtable(&storage.state_lock.lock())
        .unwrap();
    assert_eq!(storage.state.read().imm_memtables.len(), 1);
    let previous_approximate_size = storage.state.read().imm_memtables[0].approximate_size();
    assert!(previous_approximate_size >= 15);
    storage.put(b"1", b"2333").unwrap();
    storage.put(b"2", b"23333").unwrap();
    storage.put(b"3", b"233333").unwrap();
    storage
        .force_freeze_memtable(&storage.state_lock.lock())
        .unwrap();
    assert_eq!(storage.state.read().imm_memtables.len(), 2);
    assert!(
        storage.state.read().imm_memtables[1].approximate_size() == previous_approximate_size,
        "memtable 顺序错误？"
    );
    assert!(storage.state.read().imm_memtables[0].approximate_size() > previous_approximate_size);
}

#[test]
fn test_task3_freeze_on_capacity() {
    let dir = tempdir().unwrap();
    let mut options = LsmStorageOptions::default_for_week1_test();
    options.target_sst_size = 1024;
    options.num_memtable_limit = 1000;
    let storage = Arc::new(LsmStorageInner::open(dir.path(), options).unwrap());
    for _ in 0..1000 {
        storage.put(b"1", b"2333").unwrap();
    }
    let num_imm_memtables = storage.state.read().imm_memtables.len();
    assert!(num_imm_memtables >= 1, "没有 memtable 被冻结？");
    for _ in 0..1000 {
        storage.delete(b"1").unwrap();
    }
    assert!(
        storage.state.read().imm_memtables.len() > num_imm_memtables,
        "没有更多 memtable 被冻结？"
    );
}

#[test]
fn test_task4_storage_integration() {
    let dir = tempdir().unwrap();
    let storage = Arc::new(
        LsmStorageInner::open(dir.path(), LsmStorageOptions::default_for_week1_test()).unwrap(),
    );
    assert_eq!(&storage.get(b"0").unwrap(), &None);
    storage.put(b"1", b"233").unwrap();
    storage.put(b"2", b"2333").unwrap();
    storage.put(b"3", b"23333").unwrap();
    storage
        .force_freeze_memtable(&storage.state_lock.lock())
        .unwrap();
    storage.delete(b"1").unwrap();
    storage.delete(b"2").unwrap();
    storage.put(b"3", b"2333").unwrap();
    storage.put(b"4", b"23333").unwrap();
    storage
        .force_freeze_memtable(&storage.state_lock.lock())
        .unwrap();
    storage.put(b"1", b"233333").unwrap();
    storage.put(b"3", b"233333").unwrap();
    assert_eq!(storage.state.read().imm_memtables.len(), 2);
    assert_eq!(&storage.get(b"1").unwrap().unwrap()[..], b"233333");
    assert_eq!(&storage.get(b"2").unwrap(), &None);
    assert_eq!(&storage.get(b"3").unwrap().unwrap()[..], b"233333");
    assert_eq!(&storage.get(b"4").unwrap().unwrap()[..], b"23333");
}
