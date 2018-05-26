# hxo-parser-template

Template parser for HXO framework

## 目的 (Purpose)

`hxo-parser-template` 专门用于解析 HXO 文件的 `<template>` 块。它将类 HTML 的标记语言转换为结构化的模板 AST，识别元素、文本、插值表达式和指令（如 `:class`, `@click`）。

## 功能 (Features)

- **HTML 兼容解析**: 支持标准的 HTML 标签和属性结构。
- **插值识别**: 自动识别 `{{ ... }}` 格式的插值表达式。
- **指令系统**: 识别并分类以 `:`, `@`, `v-` 开头的指令。
- **自闭合标签**: 正确处理自闭合标签（如 `<img />`）和空元素。

## 维护指南 (Maintenance Guide)

### 核心逻辑
- `TemplateParser` ([lib.rs](file:///e:/模板引擎/project-hxo/compilers/hxo-parser-template/src/lib.rs)): 模板解析的主入口。
- `TemplateNode`: 节点枚举（Element, Text, Interpolation, Comment）。
- `parse_element`: 处理标签、属性、指令和子节点的递归方法。

### 依赖项
- `html5ever`/`markup5ever_rcdom`: （可选/计划中）用于增强 HTML 解析的健壮性。
- `hxo-types`: 提供位置追踪。

### 测试
- 运行 `cargo test -p hxo-parser-template`。
- 测试用例应覆盖嵌套标签、插值、指令和非正常闭合的情况。
