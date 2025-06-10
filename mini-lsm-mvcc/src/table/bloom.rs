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

// 版权所有 2021 TiKV Project Authors. 根据 Apache-2.0 获得许可。

use anyhow::{Result, bail};
use bytes::{Buf, BufMut, Bytes, BytesMut};

/// 实现一个布隆过滤器
pub struct Bloom {
    ///过滤器的位数据
    pub(crate) filter: Bytes,
    /// 哈希函数的数量
    pub(crate) k: u8,
}

pub trait BitSlice {
    fn get_bit(&self, idx: usize) -> bool;
    fn bit_len(&self) -> usize;
}

pub trait BitSliceMut {
    fn set_bit(&mut self, idx: usize, val: bool);
}

impl<T: AsRef<[u8]>> BitSlice for T {
    fn get_bit(&self, idx: usize) -> bool {
        let pos = idx / 8;
        let offset = idx % 8;
        (self.as_ref()[pos] & (1 << offset)) != 0
    }

    fn bit_len(&self) -> usize {
        self.as_ref().len() * 8
    }
}

impl<T: AsMut<[u8]>> BitSliceMut for T {
    fn set_bit(&mut self, idx: usize, val: bool) {
        let pos = idx / 8;
        let offset = idx % 8;
        if val {
            self.as_mut()[pos] |= 1 << offset;
        } else {
            self.as_mut()[pos] &= !(1 << offset);
        }
    }
}

impl Bloom {
    /// 解码布隆过滤器
    pub fn decode(buf: &[u8]) -> Result<Self> {
        let checksum = (&buf[buf.len() - 4..buf.len()]).get_u32();
        if checksum != crc32fast::hash(&buf[..buf.len() - 4]) {
            bail!("布隆过滤器校验和不匹配");
        }
        let filter = &buf[..buf.len() - 5];
        let k = buf[buf.len() - 5];
        Ok(Self {
            filter: filter.to_vec().into(),
            k,
        })
    }

    /// 编码布隆过滤器
    pub fn encode(&self, buf: &mut Vec<u8>) {
        let offset = buf.len();
        buf.extend(&self.filter);
        buf.put_u8(self.k);
        let checksum = crc32fast::hash(&buf[offset..]);
        buf.put_u32(checksum);
    }

    /// 从条目数和误报率获取每个键的布隆过滤器位数
    pub fn bloom_bits_per_key(entries: usize, false_positive_rate: f64) -> usize {
        let size =
            -1.0 * (entries as f64) * false_positive_rate.ln() / std::f64::consts::LN_2.powi(2);
        let locs = (size / (entries as f64)).ceil();
        locs as usize
    }

    /// 从键哈希构建布隆过滤器
    pub fn build_from_key_hashes(keys: &[u32], bits_per_key: usize) -> Self {
        let k = (bits_per_key as f64 * 0.69) as u32;
        let k = k.clamp(1, 30);
        let nbits = (keys.len() * bits_per_key).max(64);
        let nbytes = (nbits + 7) / 8;
        let nbits = nbytes * 8;
        let mut filter = BytesMut::with_capacity(nbytes);
        filter.resize(nbytes, 0);
        for h in keys {
            let mut h = *h;
            let delta = h.rotate_left(15);
            for _ in 0..k {
                let bit_pos = (h as usize) % nbits;
                filter.set_bit(bit_pos, true);
                h = h.wrapping_add(delta);
            }
        }
        Self {
            filter: filter.freeze(),
            k: k as u8,
        }
    }

    /// 检查布隆过滤器是否可能包含某些数据
    pub fn may_contain(&self, mut h: u32) -> bool {
        if self.k > 30 {
            // 短布隆过滤器的潜在新编码
            true
        } else {
            let nbits = self.filter.bit_len();
            let delta = h.rotate_left(15);
            for _ in 0..self.k {
                let bit_pos = h % (nbits as u32);
                if !self.filter.get_bit(bit_pos as usize) {
                    return false;
                }
                h = h.wrapping_add(delta);
            }
            true
        }
    }
}
