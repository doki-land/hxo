# hxo-parser-sass

SASS parser for HXO framework

## 目的 (Purpose)

`hxo-parser-sass` 用于处理 HXO 文件中带有 `lang="sass"` 属性的样式块（缩进语法）。

## 功能 (Features)

- **SASS 编译**: 支持传统的 SASS 缩进语法编译。

## 维护指南 (Maintenance Guide)

### 核心逻辑
- 封装 `grass` 的 SASS 语法处理 ([lib.rs](file:///e:/模板引擎/project-hxo/compilers/hxo-parser-sass/src/lib.rs))。

### 依赖项
- `grass`: 支持 SCSS/SASS 的 Rust 库。

### 测试
- 运行 `cargo test -p hxo-parser-sass`。
