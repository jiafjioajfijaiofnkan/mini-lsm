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

use tempfile::tempdir;

use crate::{
    key::{KeySlice, TS_ENABLED},
    table::{FileObject, SsTable, SsTableBuilder, bloom::Bloom},
};

fn key_of(idx: usize) -> Vec<u8> {
    format!("key_{:010}", idx * 5).into_bytes()
}

fn value_of(idx: usize) -> Vec<u8> {
    format!("value_{:010}", idx).into_bytes()
}

fn num_of_keys() -> usize {
    100
}

#[test]
fn test_task1_bloom_filter() {
    let mut key_hashes = Vec::new();
    for idx in 0..num_of_keys() {
        let key = key_of(idx);
        key_hashes.push(farmhash::fingerprint32(&key));
    }
    let bits_per_key = Bloom::bloom_bits_per_key(key_hashes.len(), 0.01);
    println!("每个键的位数: {}", bits_per_key);
    let bloom = Bloom::build_from_key_hashes(&key_hashes, bits_per_key);
    println!("布隆过滤器大小: {}, k={}", bloom.filter.len(), bloom.k);
    assert!(bloom.k < 30);
    for idx in 0..num_of_keys() {
        let key = key_of(idx);
        assert!(bloom.may_contain(farmhash::fingerprint32(&key)));
    }
    let mut x = 0;
    let mut cnt = 0;
    for idx in num_of_keys()..(num_of_keys() * 10) {
        let key = key_of(idx);
        if bloom.may_contain(farmhash::fingerprint32(&key)) {
            x += 1;
        }
        cnt += 1;
    }
    assert_ne!(x, cnt, "布隆过滤器未生效？");
    assert_ne!(x, 0, "布隆过滤器未生效？");
}

#[test]
fn test_task2_sst_decode() {
    let mut builder = SsTableBuilder::new(128);
    for idx in 0..num_of_keys() {
        let key = key_of(idx);
        let value = value_of(idx);
        builder.add(KeySlice::for_testing_from_slice_no_ts(&key[..]), &value[..]);
    }
    let dir = tempdir().unwrap();
    let path = dir.path().join("1.sst");
    let sst = builder.build_for_test(&path).unwrap();
    let sst2 = SsTable::open(0, None, FileObject::open(&path).unwrap()).unwrap();
    let bloom_1 = sst.bloom.as_ref().unwrap();
    let bloom_2 = sst2.bloom.as_ref().unwrap();
    assert_eq!(bloom_1.k, bloom_2.k);
    assert_eq!(bloom_1.filter, bloom_2.filter);
}

#[test]
fn test_task3_block_key_compression() {
    let mut builder = SsTableBuilder::new(128);
    for idx in 0..num_of_keys() {
        let key = key_of(idx);
        let value = value_of(idx);
        builder.add(KeySlice::for_testing_from_slice_no_ts(&key[..]), &value[..]);
    }
    let dir = tempdir().unwrap();
    let path = dir.path().join("1.sst");
    let sst = builder.build_for_test(path).unwrap();
    if TS_ENABLED {
        assert!(
            sst.block_meta.len() <= 34,
            "您有 {} 个块，预期 34 个",
            sst.block_meta.len()
        );
    } else {
        assert!(
            sst.block_meta.len() <= 25,
            "您有 {} 个块，预期 25 个",
            sst.block_meta.len()
        );
    }
}
