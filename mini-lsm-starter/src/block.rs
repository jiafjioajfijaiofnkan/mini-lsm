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

mod builder;
mod iterator;

pub use builder::BlockBuilder;
use bytes::Bytes;
pub use iterator::BlockIterator;

/// 块 (block) 是 LSM 树中读取和缓存的最小单元。它是有序键值对的集合。
pub struct Block {
    pub(crate) data: Vec<u8>,
    pub(crate) offsets: Vec<u16>,
}

impl Block {
    /// 将内部数据编码为课程中说明的数据布局
    /// 注意：您可能需要重新检查您的输出中是否缺少任何预期字段
    pub fn encode(&self) -> Bytes {
        unimplemented!() // TODO: 实现此功能
    }

    /// 从数据布局解码，将输入的 `data` 转换为单个 `Block`
    pub fn decode(data: &[u8]) -> Self {
        unimplemented!() // TODO: 实现此功能
    }
}
