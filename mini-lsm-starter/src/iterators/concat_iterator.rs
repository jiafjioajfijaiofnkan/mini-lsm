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

use std::sync::Arc;

use anyhow::Result;

use super::StorageIterator;
use crate::{
    key::KeySlice,
    table::{SsTable, SsTableIterator},
};

/// 将多个按键顺序排列且键范围不重叠的迭代器连接起来。
/// 我们不希望在初始化此迭代器时创建所有迭代器，以减少寻址的开销。
pub struct SstConcatIterator {
    current: Option<SsTableIterator>,
    next_sst_idx: usize,
    sstables: Vec<Arc<SsTable>>,
}

impl SstConcatIterator {
    pub fn create_and_seek_to_first(sstables: Vec<Arc<SsTable>>) -> Result<Self> {
        unimplemented!() // TODO: 实现此功能
    }

    pub fn create_and_seek_to_key(sstables: Vec<Arc<SsTable>>, key: KeySlice) -> Result<Self> {
        unimplemented!() // TODO: 实现此功能
    }
}

impl StorageIterator for SstConcatIterator {
    type KeyType<'a> = KeySlice<'a>;

    fn key(&self) -> KeySlice {
        unimplemented!() // TODO: 实现此功能
    }

    fn value(&self) -> &[u8] {
        unimplemented!() // TODO: 实现此功能
    }

    fn is_valid(&self) -> bool {
        unimplemented!() // TODO: 实现此功能
    }

    fn next(&mut self) -> Result<()> {
        unimplemented!() // TODO: 实现此功能
    }

    fn num_active_iterators(&self) -> usize {
        1
    }
}
