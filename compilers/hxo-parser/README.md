# hxo-parser

Core parser infrastructure for HXO framework

## 目的 (Purpose)

`hxo-parser` 负责 HXO 文件的一级解析（块解析）。它将单文件组件（SFC）拆分为不同的功能块（如 `<script>`, `<template>`, `<style>`, `<metadata>`），并根据块的类型（如 `lang="ts"`）分发给相应的专用解析器进行二次处理。

## 功能 (Features)

- **SFC 分块解析**: 识别并提取 `<...>` 格式的顶层块。
- **智能分发**: 根据 `lang` 属性自动选择 `hxo-parser-typescript`, `hxo-parser-scss` 等专用解析器。
- **统一输出**: 生成 `ParsedHxoFile`，包含所有已处理块的结构化数据。
- **位置感知**: 在分块过程中保持精确的源码位置信息。

## 维护指南 (Maintenance Guide)

### 核心逻辑
- `Parser` ([lib.rs](file:///e:/模板引擎/project-hxo/compilers/hxo-parser/src/lib.rs)): 顶层解析器类，负责逐字符扫描块结构。
- `parse_all`: 核心流程方法，执行分块并调用专用解析器。
- `ParsedHxoFile`: 解析结果的聚合容器。

### 依赖项
- `hxo-types`: 基础数据类型支持。
- `hxo-parser-yaml`/`json`: 用于解析元数据块。
- `hxo-parser-typescript`: 用于处理脚本块。
- `hxo-parser-scss`: 用于处理样式块。

### 测试
- 运行 `cargo test -p hxo-parser`。
- 测试应包含各种带有不同 `lang` 属性的块组合。
