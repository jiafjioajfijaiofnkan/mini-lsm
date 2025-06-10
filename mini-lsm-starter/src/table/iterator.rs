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

use super::SsTable;
use crate::{block::BlockIterator, iterators::StorageIterator, key::KeySlice};

/// SSTable 内容的迭代器。
pub struct SsTableIterator {
    table: Arc<SsTable>,
    blk_iter: BlockIterator,
    blk_idx: usize,
}

impl SsTableIterator {
    /// 创建一个新的迭代器并寻找到第一个数据块中的第一个键值对。
    pub fn create_and_seek_to_first(table: Arc<SsTable>) -> Result<Self> {
        unimplemented!() // TODO: 实现此功能
    }

    /// 寻找到第一个数据块中的第一个键值对。
    pub fn seek_to_first(&mut self) -> Result<()> {
        unimplemented!() // TODO: 实现此功能
    }

    /// 创建一个新的迭代器并寻找到第一个大于等于 `key` 的键值对。
    pub fn create_and_seek_to_key(table: Arc<SsTable>, key: KeySlice) -> Result<Self> {
        unimplemented!() // TODO: 实现此功能
    }

    /// 寻找到第一个大于等于 `key` 的键值对。
    /// 注意：在实现此函数时，您可能需要查看讲义以获取详细说明。
    pub fn seek_to_key(&mut self, key: KeySlice) -> Result<()> {
        unimplemented!() // TODO: 实现此功能
    }
}

impl StorageIterator for SsTableIterator {
    type KeyType<'a> = KeySlice<'a>;

    /// 返回底层块迭代器持有的 `key`。
    fn key(&self) -> KeySlice {
        unimplemented!() // TODO: 实现此功能
    }

    /// 返回底层块迭代器持有的 `value`。
    fn value(&self) -> &[u8] {
        unimplemented!() // TODO: 实现此功能
    }

    /// 返回当前块迭代器是否有效。
    fn is_valid(&self) -> bool {
        unimplemented!() // TODO: 实现此功能
    }

    /// 移动到块中的下一个 `key`。
    /// 注意：移动后您可能需要检查当前块迭代器是否有效。
    fn next(&mut self) -> Result<()> {
        unimplemented!() // TODO: 实现此功能
    }
}
