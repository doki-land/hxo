# hxo-parser-less

Less parser for HXO framework

## 目的 (Purpose)

`hxo-parser-less` 用于处理 HXO 文件中带有 `lang="less"` 属性的样式块。

## 功能 (Features)

- **Less 支持**: 提供对 Less 语法的解析和编译能力（目前为基础实现）。

## 维护指南 (Maintenance Guide)

### 核心逻辑
- 位于 [lib.rs](file:///e:/模板引擎/project-hxo/compilers/hxo-parser-less/src/lib.rs) 的编译入口。

### 依赖项
- 依赖于 Rust 生态中的 Less 处理库（如需增强，可考虑桥接 JS 版本或寻找高性能 Rust 实现）。

### 测试
- 运行 `cargo test -p hxo-parser-less`。
