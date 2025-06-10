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

use std::path::Path;
use std::sync::Arc;

use anyhow::Result;

use super::{BlockMeta, SsTable};
use crate::{block::BlockBuilder, key::KeySlice, lsm_storage::BlockCache};

/// 从键值对构建 SSTable。
pub struct SsTableBuilder {
    builder: BlockBuilder,
    first_key: Vec<u8>,
    last_key: Vec<u8>,
    data: Vec<u8>,
    pub(crate) meta: Vec<BlockMeta>,
    block_size: usize,
}

impl SsTableBuilder {
    /// 根据目标块大小创建一个构建器。
    pub fn new(block_size: usize) -> Self {
        unimplemented!() // TODO: 实现此功能
    }

    /// 将键值对添加到 SSTable。
    ///
    /// 注意：当当前块已满时，您应该分割出一个新块。（`std::mem::replace` 可能在此处有用）
    pub fn add(&mut self, key: KeySlice, value: &[u8]) {
        unimplemented!() // TODO: 实现此功能
    }

    /// 获取 SSTable 的估计大小。
    ///
    /// 由于数据块包含的数据远多于元数据块，因此此处仅返回数据块的大小。
    pub fn estimated_size(&self) -> usize {
        unimplemented!() // TODO: 实现此功能
    }

    /// 构建 SSTable 并将其写入给定路径。使用 `FileObject` 结构来操作磁盘对象。
    pub fn build(
        mut self,
        id: usize,
        block_cache: Option<Arc<BlockCache>>,
        path: impl AsRef<Path>,
    ) -> Result<SsTable> {
        unimplemented!() // TODO: 实现此功能
    }

    #[cfg(test)]
    pub(crate) fn build_for_test(self, path: impl AsRef<Path>) -> Result<SsTable> {
        self.build(0, None, path)
    }
}
