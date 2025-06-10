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

use bytes::BufMut;

use crate::key::{KeySlice, KeyVec};

use super::{Block, SIZEOF_U16};

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

fn compute_overlap(first_key: KeySlice, key: KeySlice) -> usize {
    let mut i = 0;
    loop {
        if i >= first_key.len() || i >= key.len() {
            break;
        }
        if first_key.raw_ref()[i] != key.raw_ref()[i] {
            break;
        }
        i += 1;
    }
    i
}

impl BlockBuilder {
    /// 创建一个新的块构建器。
    pub fn new(block_size: usize) -> Self {
        Self {
            offsets: Vec::new(),
            data: Vec::new(),
            block_size,
            first_key: KeyVec::new(),
        }
    }

    fn estimated_size(&self) -> usize {
        SIZEOF_U16 /* 块中的键值对数量 */ +  self.offsets.len() * SIZEOF_U16 /* 偏移量 */ + self.data.len()
        // 键值对
    }

    /// 向块中添加一个键值对。当块已满时返回 false。
    #[must_use]
    pub fn add(&mut self, key: KeySlice, value: &[u8]) -> bool {
        assert!(!key.is_empty(), "键不能为空");
        if self.estimated_size() + key.len() + value.len() + SIZEOF_U16 * 3 /* key_len, value_len 和 offset */ > self.block_size
            && !self.is_empty()
        {
            return false;
        }
        // 将数据的偏移量添加到偏移量数组中。
        self.offsets.push(self.data.len() as u16);
        let overlap = compute_overlap(self.first_key.as_key_slice(), key);
        // 编码键的重叠部分。
        self.data.put_u16(overlap as u16);
        // 编码键的长度。
        self.data.put_u16((key.len() - overlap) as u16);
        // 编码键的内容。
        self.data.put(&key.raw_ref()[overlap..]);
        // 编码值的长度。
        self.data.put_u16(value.len() as u16);
        // 编码值的内容。
        self.data.put(value);

        if self.first_key.is_empty() {
            self.first_key = key.to_key_vec();
        }

        true
    }

    /// 检查块中是否没有键值对。
    pub fn is_empty(&self) -> bool {
        self.offsets.is_empty()
    }

    /// 完成块的构建。
    pub fn build(self) -> Block {
        if self.is_empty() {
            panic!("块不应为空");
        }
        Block {
            data: self.data,
            offsets: self.offsets,
        }
    }
}
