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

mod builder;
mod iterator;

pub use builder::BlockBuilder;
use bytes::{Buf, BufMut, Bytes};
pub use iterator::BlockIterator;

pub(crate) const SIZEOF_U16: usize = std::mem::size_of::<u16>();

/// 块 (block) 是 LSM 树中读取和缓存的最小单元。它是有序键值对的集合。
pub struct Block {
    pub(crate) data: Vec<u8>,
    pub(crate) offsets: Vec<u16>,
}

impl Block {
    pub fn encode(&self) -> Bytes {
        let mut buf = self.data.clone();
        let offsets_len = self.offsets.len();
        for offset in &self.offsets {
            buf.put_u16(*offset);
        }
        // 在块的末尾添加元素数量
        buf.put_u16(offsets_len as u16);
        buf.into()
    }

    pub fn decode(data: &[u8]) -> Self {
        // 获取块中元素的数量
        let entry_offsets_len = (&data[data.len() - SIZEOF_U16..]).get_u16() as usize;
        let data_end = data.len() - SIZEOF_U16 - entry_offsets_len * SIZEOF_U16;
        let offsets_raw = &data[data_end..data.len() - SIZEOF_U16];
        // 获取偏移量数组
        let offsets = offsets_raw
            .chunks(SIZEOF_U16)
            .map(|mut x| x.get_u16())
            .collect();
        // 检索数据
        let data = data[0..data_end].to_vec();
        Self { data, offsets }
    }
}
