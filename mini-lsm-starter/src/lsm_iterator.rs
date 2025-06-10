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

use anyhow::Result;

use crate::{
    iterators::{StorageIterator, merge_iterator::MergeIterator},
    mem_table::MemTableIterator,
};

/// 表示 LSM 迭代器的内部类型。此类型将在课程中多次更改。
type LsmIteratorInner = MergeIterator<MemTableIterator>;

pub struct LsmIterator {
    inner: LsmIteratorInner,
}

impl LsmIterator {
    pub(crate) fn new(iter: LsmIteratorInner) -> Result<Self> {
        Ok(Self { inner: iter })
    }
}

impl StorageIterator for LsmIterator {
    type KeyType<'a> = &'a [u8];

    fn is_valid(&self) -> bool {
        unimplemented!() // TODO: 实现此功能
    }

    fn key(&self) -> &[u8] {
        unimplemented!() // TODO: 实现此功能
    }

    fn value(&self) -> &[u8] {
        unimplemented!() // TODO: 实现此功能
    }

    fn next(&mut self) -> Result<()> {
        unimplemented!() // TODO: 实现此功能
    }
}

/// 一个围绕现有迭代器的包装器，当迭代器无效时，将阻止用户调用 `next`。
/// 如果迭代器已经无效，`next` 不执行任何操作。如果 `next` 返回错误，
/// `is_valid` 应返回 false，并且 `next` 应始终返回错误。
pub struct FusedIterator<I: StorageIterator> {
    iter: I,
    has_errored: bool,
}

impl<I: StorageIterator> FusedIterator<I> {
    pub fn new(iter: I) -> Self {
        Self {
            iter,
            has_errored: false,
        }
    }
}

impl<I: StorageIterator> StorageIterator for FusedIterator<I> {
    type KeyType<'a>
        = I::KeyType<'a>
    where
        Self: 'a;

    fn is_valid(&self) -> bool {
        unimplemented!() // TODO: 实现此功能
    }

    fn key(&self) -> Self::KeyType<'_> {
        unimplemented!() // TODO: 实现此功能
    }

    fn value(&self) -> &[u8] {
        unimplemented!() // TODO: 实现此功能
    }

    fn next(&mut self) -> Result<()> {
        unimplemented!() // TODO: 实现此功能
    }
}
