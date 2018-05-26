# hxo-lsp

Language Server Protocol implementation for HXO

## 目的 (Purpose)

`hxo-lsp` 为 HXO 框架提供语言服务器实现，旨在为 IDE（如 VS Code）提供语法高亮、自动补全、错误诊断和定义跳转等开发体验增强功能。

## 功能 (Features)

- **实时诊断**: 基于编译器输出提供语法错误反馈。
- **自动补全**: （开发中）提供标签、属性和组件名的智能提示。
- **文档同步**: 支持 LSP 标准的文档打开、修改和关闭通知。

## 维护指南 (Maintenance Guide)

### 核心逻辑
- 基于 `tower-lsp` 的服务器循环 ([lib.rs](file:///e:/模板引擎/project-hxo/compilers/hxo-lsp/src/lib.rs))。

### 依赖项
- `tower-lsp`: 强大的 LSP 服务器框架。
- `hxo-compiler`: 用于在后台运行编译以获取诊断信息。

### 测试
- 运行 `cargo test -p hxo-lsp`。
