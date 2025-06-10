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

use std::fs::{File, OpenOptions};
use std::hash::Hasher;
use std::io::{BufWriter, Read, Write};
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result, bail};
use bytes::{Buf, BufMut, Bytes};
use crossbeam_skiplist::SkipMap;
use parking_lot::Mutex;

use crate::key::{KeyBytes, KeySlice};

pub struct Wal {
    file: Arc<Mutex<BufWriter<File>>>,
}

impl Wal {
    pub fn create(path: impl AsRef<Path>) -> Result<Self> {
        Ok(Self {
            file: Arc::new(Mutex::new(BufWriter::new(
                OpenOptions::new()
                    .read(true)
                    .create_new(true)
                    .write(true)
                    .open(path)
                    .context("无法创建 WAL 文件")?,
            ))),
        })
    }

    pub fn recover(path: impl AsRef<Path>, skiplist: &SkipMap<KeyBytes, Bytes>) -> Result<Self> {
        let path = path.as_ref();
        let mut file = OpenOptions::new()
            .read(true)
            .append(true)
            .open(path)
            .context("无法从 WAL 文件恢复")?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        let mut rbuf: &[u8] = buf.as_slice();
        while rbuf.has_remaining() {
            let batch_size = rbuf.get_u32() as usize;
            if rbuf.remaining() < batch_size {
                bail!("不完整的 WAL 文件");
            }
            let mut batch_buf = &rbuf[..batch_size];
            let mut kv_pairs = Vec::new();
            let mut hasher = crc32fast::Hasher::new();
            // 从各个组件计算出的校验和应与直接对缓冲区进行校验和计算的结果相同。
            // 学生的实现只需要对缓冲区进行一次校验和计算。我们计算两者是为了验证目的。
            let single_checksum = crc32fast::hash(batch_buf);
            while batch_buf.has_remaining() {
                let key_len = batch_buf.get_u16() as usize;
                hasher.write_u16(key_len as u16);
                let key = Bytes::copy_from_slice(&batch_buf[..key_len]);
                hasher.write(&key);
                batch_buf.advance(key_len);
                let ts = batch_buf.get_u64();
                hasher.write(&ts.to_be_bytes());
                let value_len = batch_buf.get_u16() as usize;
                hasher.write_u16(value_len as u16);
                let value = Bytes::copy_from_slice(&batch_buf[..value_len]);
                hasher.write(&value);
                kv_pairs.push((key, ts, value));
                batch_buf.advance(value_len);
            }
            rbuf.advance(batch_size);
            let expected_checksum = rbuf.get_u32();
            let component_checksum = hasher.finalize();
            assert_eq!(component_checksum, single_checksum);
            if single_checksum != expected_checksum {
                bail!("校验和不匹配");
            }
            for (key, ts, value) in kv_pairs {
                skiplist.insert(KeyBytes::from_bytes_with_ts(key, ts), value);
            }
        }
        Ok(Self {
            file: Arc::new(Mutex::new(BufWriter::new(file))),
        })
    }

    /// 在第 3 周第 5 天实现此功能。
    pub fn put_batch(&self, data: &[(KeySlice, &[u8])]) -> Result<()> {
        let mut file = self.file.lock();
        let mut buf = Vec::<u8>::new();
        for (key, value) in data {
            buf.put_u16(key.key_len() as u16);
            buf.put_slice(key.key_ref());
            buf.put_u64(key.ts());
            buf.put_u16(value.len() as u16);
            buf.put_slice(value);
        }
        // 写入批处理大小头部 (u32)
        file.write_all(&(buf.len() as u32).to_be_bytes())?;
        // 写入键值对主体
        file.write_all(&buf)?;
        // 写入校验和 (u32)
        file.write_all(&crc32fast::hash(&buf).to_be_bytes())?;
        Ok(())
    }

    pub fn put(&self, key: KeySlice, value: &[u8]) -> Result<()> {
        self.put_batch(&[(key, value)])
    }

    pub fn sync(&self) -> Result<()> {
        let mut file = self.file.lock();
        file.flush()?;
        file.get_mut().sync_all()?;
        Ok(())
    }
}
