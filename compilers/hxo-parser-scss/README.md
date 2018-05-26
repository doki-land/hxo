# hxo-parser-scss

SCSS parser for HXO framework

## 目的 (Purpose)

`hxo-parser-scss` 用于处理 HXO 文件中带有 `lang="scss"` 属性的样式块。它将 SCSS 源码编译为标准的 CSS。

## 功能 (Features)

- **SCSS 编译**: 支持变量、嵌套、混合（Mixins）等 SCSS 特性。
- **配置支持**: 支持传递编译选项（如输出格式、包含路径）。

## 维护指南 (Maintenance Guide)

### 核心逻辑
- `compile` 函数 ([lib.rs](file:///e:/模板引擎/project-hxo/compilers/hxo-parser-scss/src/lib.rs)): 使用 `grass` 库执行同步编译。

### 依赖项
- `grass`: 纯 Rust 实现的 SCSS 编译器。

### 测试
- 运行 `cargo test -p hxo-parser-scss`。
- 验证嵌套规则和变量替换是否正确。
