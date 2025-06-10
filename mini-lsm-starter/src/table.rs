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

pub(crate) mod bloom;
mod builder;
mod iterator;

use std::fs::File;
use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
pub use builder::SsTableBuilder;
use bytes::Buf;
pub use iterator::SsTableIterator;

use crate::block::Block;
use crate::key::{KeyBytes, KeySlice};
use crate::lsm_storage::BlockCache;

use self::bloom::Bloom;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockMeta {
    /// 此数据块的偏移量。
    pub offset: usize,
    /// 数据块的第一个键。
    pub first_key: KeyBytes,
    /// 数据块的最后一个键。
    pub last_key: KeyBytes,
}

impl BlockMeta {
    /// 将块元数据编码到缓冲区。
    /// 您可能需要向缓冲区添加额外的字段，
    /// 以便将来从同一缓冲区解码时帮助跟踪 `first_key`。
    pub fn encode_block_meta(
        block_meta: &[BlockMeta],
        #[allow(clippy::ptr_arg)] // 完成后移除此 allow
        buf: &mut Vec<u8>,
    ) {
        unimplemented!() // TODO: 实现此功能
    }

    /// 从缓冲区解码块元数据。
    pub fn decode_block_meta(buf: impl Buf) -> Vec<BlockMeta> {
        unimplemented!() // TODO: 实现此功能
    }
}

/// 文件对象。
pub struct FileObject(Option<File>, u64);

impl FileObject {
    pub fn read(&self, offset: u64, len: u64) -> Result<Vec<u8>> {
        use std::os::unix::fs::FileExt;
        let mut data = vec![0; len as usize];
        self.0
            .as_ref()
            .unwrap()
            .read_exact_at(&mut data[..], offset)?;
        Ok(data)
    }

    pub fn size(&self) -> u64 {
        self.1
    }

    /// 创建一个新的文件对象（第 2 天）并将文件写入磁盘（第 4 天）。
    pub fn create(path: &Path, data: Vec<u8>) -> Result<Self> {
        std::fs::write(path, &data)?;
        File::open(path)?.sync_all()?;
        Ok(FileObject(
            Some(File::options().read(true).write(false).open(path)?),
            data.len() as u64,
        ))
    }

    pub fn open(path: &Path) -> Result<Self> {
        let file = File::options().read(true).write(false).open(path)?;
        let size = file.metadata()?.len();
        Ok(FileObject(Some(file), size))
    }
}

/// SSTable。
pub struct SsTable {
    /// SSTable 的实际存储单元，格式如上所述。
    pub(crate) file: FileObject,
    /// 保存数据块信息的元数据块。
    pub(crate) block_meta: Vec<BlockMeta>,
    /// 指示 `file` 中元数据块起点的偏移量。
    pub(crate) block_meta_offset: usize,
    id: usize,
    block_cache: Option<Arc<BlockCache>>,
    first_key: KeyBytes,
    last_key: KeyBytes,
    pub(crate) bloom: Option<Bloom>,
    /// 此 SST 中存储的最大时间戳，在第 3 周实现。
    max_ts: u64,
}

impl SsTable {
    #[cfg(test)]
    pub(crate) fn open_for_test(file: FileObject) -> Result<Self> {
        Self::open(0, None, file)
    }

    /// 从文件打开 SSTable。
    pub fn open(id: usize, block_cache: Option<Arc<BlockCache>>, file: FileObject) -> Result<Self> {
        unimplemented!() // TODO: 实现此功能
    }

    /// 创建一个仅包含第一个键 + 最后一个键元数据的模拟 SST
    pub fn create_meta_only(
        id: usize,
        file_size: u64,
        first_key: KeyBytes,
        last_key: KeyBytes,
    ) -> Self {
        Self {
            file: FileObject(None, file_size),
            block_meta: vec![],
            block_meta_offset: 0,
            id,
            block_cache: None,
            first_key,
            last_key,
            bloom: None,
            max_ts: 0,
        }
    }

    /// 从磁盘读取一个块。
    pub fn read_block(&self, block_idx: usize) -> Result<Arc<Block>> {
        unimplemented!() // TODO: 实现此功能
    }

    /// 从磁盘读取一个块，使用块缓存。（第 4 天）
    pub fn read_block_cached(&self, block_idx: usize) -> Result<Arc<Block>> {
        unimplemented!() // TODO: 实现此功能
    }

    /// 找到可能包含 `key` 的块。
    /// 注意：您可能需要使用存储在 `BlockMeta` 中的 `first_key`。
    /// 您也可以假设每个连续块中存储的键值对是已排序的。
    pub fn find_block_idx(&self, key: KeySlice) -> usize {
        unimplemented!() // TODO: 实现此功能
    }

    /// 获取数据块的数量。
    pub fn num_of_blocks(&self) -> usize {
        self.block_meta.len()
    }

    pub fn first_key(&self) -> &KeyBytes {
        &self.first_key
    }

    pub fn last_key(&self) -> &KeyBytes {
        &self.last_key
    }

    pub fn table_size(&self) -> u64 {
        self.file.1
    }

    pub fn sst_id(&self) -> usize {
        self.id
    }

    pub fn max_ts(&self) -> u64 {
        self.max_ts
    }
}
