<!--
  mini-lsm-book © 2022-2025 by Alex Chi Z is licensed under CC BY-NC-SA 4.0
-->

# 环境配置 (Environment Setup)

入门代码和参考解决方案可在 [https://github.com/skyzh/mini-lsm](https://github.com/skyzh/mini-lsm) 获取。

## 安装 Rust (Install Rust)

更多信息请参阅 [https://rustup.rs](https://rustup.rs)。

## 克隆仓库 (Clone the repo)

```
git clone https://github.com/skyzh/mini-lsm
```

## 入门代码 (Starter code)

```
cd mini-lsm/mini-lsm-starter
code .
```

## 安装工具 (Install Tools)

您将需要最新稳定版的 Rust 来编译此项目。最低要求是 `1.74`。

```
cargo x install-tools
```

## 运行测试 (Run tests)

```
cargo x copy-test --week 1 --day 1
cargo x scheck
```

现在，您可以继续开始 [第 1 周：Mini-LSM](./week1-overview.md)。

{{#include copyright.md}}
