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

use bytes::Bytes;
use tempfile::tempdir;

use crate::key::KeySlice;
use crate::table::{FileObject, SsTable, SsTableBuilder, SsTableIterator};

use super::harness::{check_iter_result_by_key_and_ts, generate_sst_with_ts};

#[test]
fn test_sst_build_multi_version_simple() {
    let mut builder = SsTableBuilder::new(16);
    builder.add(
        KeySlice::for_testing_from_slice_with_ts(b"233", 233),
        b"233333",
    );
    builder.add(
        KeySlice::for_testing_from_slice_with_ts(b"233", 0),
        b"2333333",
    );
    let dir = tempdir().unwrap();
    builder.build_for_test(dir.path().join("1.sst")).unwrap();
}

fn generate_test_data() -> Vec<((Bytes, u64), Bytes)> {
    (0..100)
        .map(|id| {
            (
                (Bytes::from(format!("key{:05}", id / 5)), 5 - (id % 5)),
                Bytes::from(format!("value{:05}", id)),
            )
        })
        .collect()
}

#[test]
fn test_sst_build_multi_version_hard() {
    let dir = tempdir().unwrap();
    let data = generate_test_data();
    generate_sst_with_ts(1, dir.path().join("1.sst"), data.clone(), None);
    let sst = Arc::new(
        SsTable::open(
            1,
            None,
            FileObject::open(&dir.path().join("1.sst")).unwrap(),
        )
        .unwrap(),
    );
    check_iter_result_by_key_and_ts(
        &mut SsTableIterator::create_and_seek_to_first(sst).unwrap(),
        data,
    );
}
