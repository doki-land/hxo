# hxo-parser-css

CSS parser and optimizer for HXO framework

## 目的 (Purpose)

`hxo-parser-css` 负责解析、转换和优化 HXO 模块中的标准 CSS 代码。它通常用于处理 `<style>` 块（当没有指定预处理器时）以及编译器生成的最终 CSS。

## 功能 (Features)

- **标准 CSS 解析**: 解析标准的 CSS 语法。
- **优化与压缩**: 利用 LightningCSS 进行高效的样式压缩和浏览器前缀自动补全。
- **Scoped CSS**: （计划中）支持作用域样式转换，防止样式污染。

## 维护指南 (Maintenance Guide)

### 核心逻辑
- 封装 LightningCSS 的调用接口 ([lib.rs](file:///e:/模板引擎/project-hxo/compilers/hxo-parser-css/src/lib.rs))。

### 依赖项
- `lightningcss`: 高性能的 CSS 解析和优化库。

### 测试
- 运行 `cargo test -p hxo-parser-css`。
- 验证样式压缩和属性转换的正确性。
