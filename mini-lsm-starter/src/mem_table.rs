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

#![allow(unused_variables)] // TODO(you): 实现此模块后移除此 lint
#![allow(dead_code)] // TODO(you): 实现此模块后移除此 lint

use std::ops::Bound;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;

use anyhow::Result;
use bytes::Bytes;
use crossbeam_skiplist::SkipMap;
use ouroboros::self_referencing;

use crate::iterators::StorageIterator;
use crate::key::KeySlice;
use crate::table::SsTableBuilder;
use crate::wal::Wal;

/// 基于 crossbeam-skiplist 的基本 memtable。
///
/// memtable 的初始实现是第 1 周第 1 天的一部分。它将在第 1 周和第 2 周的其他章节中逐步实现。
pub struct MemTable {
    map: Arc<SkipMap<Bytes, Bytes>>,
    wal: Option<Wal>,
    id: usize,
    approximate_size: Arc<AtomicUsize>,
}

/// 从 `&[u8]` 的边界创建一个 `Bytes` 的边界。
pub(crate) fn map_bound(bound: Bound<&[u8]>) -> Bound<Bytes> {
    match bound {
        Bound::Included(x) => Bound::Included(Bytes::copy_from_slice(x)),
        Bound::Excluded(x) => Bound::Excluded(Bytes::copy_from_slice(x)),
        Bound::Unbounded => Bound::Unbounded,
    }
}

impl MemTable {
    /// 创建一个新的 memtable。
    pub fn create(_id: usize) -> Self {
        unimplemented!() // TODO: 实现此功能
    }

    /// 创建一个新的带 WAL 的 memtable
    pub fn create_with_wal(_id: usize, _path: impl AsRef<Path>) -> Result<Self> {
        unimplemented!() // TODO: 实现此功能
    }

    /// 从 WAL 创建一个 memtable
    pub fn recover_from_wal(_id: usize, _path: impl AsRef<Path>) -> Result<Self> {
        unimplemented!() // TODO: 实现此功能
    }

    pub fn for_testing_put_slice(&self, key: &[u8], value: &[u8]) -> Result<()> {
        self.put(key, value)
    }

    pub fn for_testing_get_slice(&self, key: &[u8]) -> Option<Bytes> {
        self.get(key)
    }

    pub fn for_testing_scan_slice(
        &self,
        lower: Bound<&[u8]>,
        upper: Bound<&[u8]>,
    ) -> MemTableIterator {
        // 此函数仅在第 1 周的测试中使用，因此在第 3 周的 key-ts 重构期间，您无需考虑边界排除/包含逻辑。
        // 只需为 key-ts 对提供 `DEFAULT_TS` 作为时间戳即可。
        self.scan(lower, upper)
    }

    /// 通过键获取值。
    pub fn get(&self, _key: &[u8]) -> Option<Bytes> {
        unimplemented!() // TODO: 实现此功能
    }

    /// 将键值对放入 memtable。
    ///
    /// 在第 1 周第 1 天，只需将键值对放入 skiplist 即可。
    /// 在第 2 周第 6 天，同时将数据刷写到 WAL。
    /// 在第 3 周第 5 天，修改函数以使用批处理 API。
    pub fn put(&self, _key: &[u8], _value: &[u8]) -> Result<()> {
        unimplemented!() // TODO: 实现此功能
    }

    /// 在第 3 周第 5 天实现此功能；如果您想更早地实现此功能，请使用 `&[u8]` 作为键类型。
    pub fn put_batch(&self, _data: &[(KeySlice, &[u8])]) -> Result<()> {
        unimplemented!() // TODO: 实现此功能
    }

    pub fn sync_wal(&self) -> Result<()> {
        if let Some(ref wal) = self.wal {
            wal.sync()?;
        }
        Ok(())
    }

    /// 获取键范围的迭代器。
    pub fn scan(&self, _lower: Bound<&[u8]>, _upper: Bound<&[u8]>) -> MemTableIterator {
        unimplemented!() // TODO: 实现此功能
    }

    /// 将 memtable 刷写到 SSTable。在第 1 周第 6 天实现。
    pub fn flush(&self, _builder: &mut SsTableBuilder) -> Result<()> {
        unimplemented!() // TODO: 实现此功能
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn approximate_size(&self) -> usize {
        self.approximate_size
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// 仅在关闭数据库时使用此函数
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

type SkipMapRangeIter<'a> =
    crossbeam_skiplist::map::Range<'a, Bytes, (Bound<Bytes>, Bound<Bytes>), Bytes, Bytes>;

/// `SkipMap` 范围的迭代器。这是一个自引用结构，更多信息请参阅第 1 周第 2 天的章节。
///
/// 这是第 1 周第 2 天的一部分。
#[self_referencing]
pub struct MemTableIterator {
    /// 存储对 skiplist 的引用。
    map: Arc<SkipMap<Bytes, Bytes>>,
    /// 存储一个 skiplist 迭代器，该迭代器引用 `MemTableIterator` 自身的生命周期。
    #[borrows(map)]
    #[not_covariant]
    iter: SkipMapRangeIter<'this>,
    /// 存储当前的键值对。
    item: (Bytes, Bytes),
}

impl StorageIterator for MemTableIterator {
    type KeyType<'a> = KeySlice<'a>;

    fn value(&self) -> &[u8] {
        unimplemented!() // TODO: 实现此功能
    }

    fn key(&self) -> KeySlice {
        unimplemented!() // TODO: 实现此功能
    }

    fn is_valid(&self) -> bool {
        unimplemented!() // TODO: 实现此功能
    }

    fn next(&mut self) -> Result<()> {
        unimplemented!() // TODO: 实现此功能
    }
}
