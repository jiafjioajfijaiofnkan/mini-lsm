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

use super::StorageIterator;

/// 将两个不同类型的迭代器合并为一个。如果两个迭代器具有相同的键，
/// 则只生成一次该键，并优先选择 A 中的条目。
pub struct TwoMergeIterator<A: StorageIterator, B: StorageIterator> {
    a: A,
    b: B,
    // 根据需要添加字段
}

impl<
    A: 'static + StorageIterator,
    B: 'static + for<'a> StorageIterator<KeyType<'a> = A::KeyType<'a>>,
> TwoMergeIterator<A, B>
{
    pub fn create(a: A, b: B) -> Result<Self> {
        unimplemented!() // TODO: 实现此功能
    }
}

impl<
    A: 'static + StorageIterator,
    B: 'static + for<'a> StorageIterator<KeyType<'a> = A::KeyType<'a>>,
> StorageIterator for TwoMergeIterator<A, B>
{
    type KeyType<'a> = A::KeyType<'a>;

    fn key(&self) -> Self::KeyType<'_> {
        unimplemented!() // TODO: 实现此功能
    }

    fn value(&self) -> &[u8] {
        unimplemented!() // TODO: 实现此功能
    }

    fn is_valid(&self) -> bool {
        unimplemented!() // TODO: 实现此功能
    }

    fn next(&mut self) -> Result<()> {
        unimplemented!() // TODO: 实现此功能
    }
}
