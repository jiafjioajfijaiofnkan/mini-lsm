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

use bytes::Buf;

use crate::{
    block::SIZEOF_U16,
    key::{KeySlice, KeyVec},
};

use super::Block;

/// 迭代一个块 (block)。
pub struct BlockIterator {
    /// 对块的引用
    block: Arc<Block>,
    /// 迭代器位置的当前键
    key: KeyVec,
    /// 块数据中当前值的范围，对应于当前键
    value_range: (usize, usize),
    /// 迭代器位置的当前索引
    idx: usize,
    /// 块中的第一个键
    first_key: KeyVec,
}

impl Block {
    fn get_first_key(&self) -> KeyVec {
        let mut buf = &self.data[..];
        buf.get_u16();
        let key_len = buf.get_u16() as usize;
        let key = &buf[..key_len];
        buf.advance(key_len);
        KeyVec::from_vec_with_ts(key.to_vec(), buf.get_u64())
    }
}

impl BlockIterator {
    fn new(block: Arc<Block>) -> Self {
        Self {
            first_key: block.get_first_key(),
            block,
            key: KeyVec::new(),
            value_range: (0, 0),
            idx: 0,
        }
    }

    /// 创建一个块迭代器并寻找到第一个条目。
    pub fn create_and_seek_to_first(block: Arc<Block>) -> Self {
        let mut iter = Self::new(block);
        iter.seek_to_first();
        iter
    }

    /// 创建一个块迭代器并寻找到第一个大于等于 `key` 的键。
    pub fn create_and_seek_to_key(block: Arc<Block>, key: KeySlice) -> Self {
        let mut iter = Self::new(block);
        iter.seek_to_key(key);
        iter
    }

    /// 返回当前条目的键。
    pub fn key(&self) -> KeySlice {
        debug_assert!(!self.key.is_empty(), "无效的迭代器");
        self.key.as_key_slice()
    }

    /// 返回当前条目的值。
    pub fn value(&self) -> &[u8] {
        debug_assert!(!self.key.is_empty(), "无效的迭代器");
        &self.block.data[self.value_range.0..self.value_range.1]
    }

    /// 如果迭代器有效，则返回 true。
    pub fn is_valid(&self) -> bool {
        !self.key.is_empty()
    }

    /// 寻找到块中的第一个键。
    pub fn seek_to_first(&mut self) {
        self.seek_to(0);
    }

    /// 寻找到块中的第 idx 个键。
    fn seek_to(&mut self, idx: usize) {
        if idx >= self.block.offsets.len() {
            self.key.clear();
            self.value_range = (0, 0);
            return;
        }
        let offset = self.block.offsets[idx] as usize;
        self.seek_to_offset(offset);
        self.idx = idx;
    }

    /// 移动到块中的下一个键。
    pub fn next(&mut self) {
        self.idx += 1;
        self.seek_to(self.idx);
    }

    /// 寻找到指定位置并更新当前的 `key` 和 `value`
    /// 索引更新将由调用者处理
    fn seek_to_offset(&mut self, offset: usize) {
        let mut entry = &self.block.data[offset..];
        // 由于 `get_u16()` 会自动将指针向前移动 2 个字节，
        // 我们不需要手动推进它
        let overlap_len = entry.get_u16() as usize;
        let key_len = entry.get_u16() as usize;
        let key = &entry[..key_len];
        self.key.clear();
        self.key.append(&self.first_key.key_ref()[..overlap_len]);
        self.key.append(key);
        entry.advance(key_len);
        let ts = entry.get_u64();
        self.key.set_ts(ts);
        let value_len = entry.get_u16() as usize;
        // 每次更改编码时，请记住更改此设置！
        let value_offset_begin =
            offset + SIZEOF_U16 + SIZEOF_U16 + std::mem::size_of::<u64>() + key_len + SIZEOF_U16;
        let value_offset_end = value_offset_begin + value_len;
        self.value_range = (value_offset_begin, value_offset_end);
        entry.advance(value_len);
    }

    /// 寻找到第一个大于等于 `key` 的键。
    pub fn seek_to_key(&mut self, key: KeySlice) {
        let mut low = 0;
        let mut high = self.block.offsets.len();
        while low < high {
            let mid = low + (high - low) / 2;
            self.seek_to(mid);
            assert!(self.is_valid());
            match self.key().cmp(&key) {
                std::cmp::Ordering::Less => low = mid + 1,
                std::cmp::Ordering::Greater => high = mid,
                std::cmp::Ordering::Equal => return,
            }
        }
        self.seek_to(low);
    }
}
