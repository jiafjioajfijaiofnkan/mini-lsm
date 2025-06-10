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

use crate::key::{KeySlice, KeyVec};

use super::Block;

/// 迭代一个块 (block)。
pub struct BlockIterator {
    /// 内部的 `Block`，由 `Arc` 包装
    block: Arc<Block>,
    /// 当前的键，为空表示迭代器无效
    key: KeyVec,
    /// 块数据中当前值的范围，对应于当前键
    value_range: (usize, usize),
    /// 当前键值对的索引，应在 [0, num_of_elements) 范围内
    idx: usize,
    /// 块中的第一个键
    first_key: KeyVec,
}

impl BlockIterator {
    fn new(block: Arc<Block>) -> Self {
        Self {
            block,
            key: KeyVec::new(),
            value_range: (0, 0),
            idx: 0,
            first_key: KeyVec::new(),
        }
    }

    /// 创建一个块迭代器并寻找到第一个条目。
    pub fn create_and_seek_to_first(block: Arc<Block>) -> Self {
        unimplemented!() // TODO: 实现此功能
    }

    /// 创建一个块迭代器并寻找到第一个大于等于 `key` 的键。
    pub fn create_and_seek_to_key(block: Arc<Block>, key: KeySlice) -> Self {
        unimplemented!() // TODO: 实现此功能
    }

    /// 返回当前条目的键。
    pub fn key(&self) -> KeySlice {
        unimplemented!() // TODO: 实现此功能
    }

    /// 返回当前条目的值。
    pub fn value(&self) -> &[u8] {
        unimplemented!() // TODO: 实现此功能
    }

    /// 如果迭代器有效，则返回 true。
    /// 注意：您可能需要使用 `key`
    pub fn is_valid(&self) -> bool {
        unimplemented!() // TODO: 实现此功能
    }

    /// 寻找到块中的第一个键。
    pub fn seek_to_first(&mut self) {
        unimplemented!() // TODO: 实现此功能
    }

    /// 移动到块中的下一个键。
    pub fn next(&mut self) {
        unimplemented!() // TODO: 实现此功能
    }

    /// 寻找到第一个大于等于 `key` 的键。
    /// 注意：您应该假设调用者添加块中的键值对时已对其进行排序。
    pub fn seek_to_key(&mut self, key: KeySlice) {
        unimplemented!() // TODO: 实现此功能
    }
}
