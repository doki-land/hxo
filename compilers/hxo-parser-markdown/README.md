# hxo-parser-markdown

Markdown parser for HXO framework

## 目的 (Purpose)

`hxo-parser-markdown` 用于解析 HXO 模块中的 Markdown 内容，通常用于文档块或将 Markdown 转换为模板节点的场景。

## 功能 (Features)

- **GFM 支持**: 支持 GitHub Flavored Markdown。
- **AST 转换**: 将 Markdown 结构映射为 HXO 可理解的节点。

## 维护指南 (Maintenance Guide)

### 核心逻辑
- 使用 `pulldown-cmark` 进行高效解析 ([lib.rs](file:///e:/模板引擎/project-hxo/compilers/hxo-parser-markdown/src/lib.rs))。

### 依赖项
- `pulldown-cmark`: Rust 中最快且符合 CommonMark 规范的解析器。

### 测试
- 运行 `cargo test -p hxo-parser-markdown`。
