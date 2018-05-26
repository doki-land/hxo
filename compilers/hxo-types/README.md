# hxo-types

Common types and error handling for the HXO framework

## 目的 (Purpose)

`hxo-types` 是 HXO 项目的基础库，定义了整个框架共享的核心数据类型、位置信息（Span/Position）和统一的错误处理机制。它确保了编译器各个组件之间在数据传递和错误报告上的高度一致性。

## 功能 (Features)

- **位置追踪**: 提供 `Span` 和 `Position` 类型，用于在报错时精确定位源代码位置。
- **统一错误系统**: 定义了 `Error` 和 `ErrorKind`，支持 IO、解析、编译等多种错误类型。
- **文件结构模型**: 定义了 `HxoFile` 和 `HxoBlock`，用于表示 HXO 文件的多块结构（SFC）。
- **路由配置**: 提供了 `RouterConfig` 和 `Route` 类型，用于处理 HXO 的路由元数据。

## 维护指南 (Maintenance Guide)

### 核心逻辑
- `Span`/`Position` ([lib.rs](file:///e:/模板引擎/project-hxo/compilers/hxo-types/src/lib.rs)): 所有的 AST 节点都应包含位置信息。
- `Error`: 实现了 `std::error::Error` 和 `Display`，方便集成和展示。
- `HxoBlock`: 表示 HXO 文件中的一个顶层块（如 `<script>` 或 `<template>`）。

### 依赖项
- `serde`: 用于基础类型的序列化。

### 测试
- 运行 `cargo test -p hxo-types`。
- 确保错误转换逻辑（如 `From<io::Error>`）和序列化行为符合预期。
