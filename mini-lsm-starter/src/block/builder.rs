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

use crate::key::{KeySlice, KeyVec};

use super::Block;

/// 构建一个块 (block)。
pub struct BlockBuilder {
    /// 每个键值对条目的偏移量。
    offsets: Vec<u16>,
    /// 块中所有序列化的键值对。
    data: Vec<u8>,
    /// 预期的块大小。
    block_size: usize,
    /// 块中的第一个键。
    first_key: KeyVec,
}

impl BlockBuilder {
    /// 创建一个新的块构建器。
    pub fn new(block_size: usize) -> Self {
        unimplemented!() // TODO: 实现此功能
    }

    /// 向块中添加一个键值对。当块已满时返回 false。
    /// 您可能会发现 `bytes::BufMut` trait 对于操作二进制数据很有用。
    #[must_use]
    pub fn add(&mut self, key: KeySlice, value: &[u8]) -> bool {
        unimplemented!() // TODO: 实现此功能
    }

    /// 检查块中是否没有键值对。
    pub fn is_empty(&self) -> bool {
        unimplemented!() // TODO: 实现此功能
    }

    /// 完成块的构建。
    pub fn build(self) -> Block {
        unimplemented!() // TODO: 实现此功能
    }
}
