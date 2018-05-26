# hxo-parser-tailwind

Tailwind CSS parser for HXO framework

## 目的 (Purpose)

`hxo-parser-tailwind` 是 HXO 编译器中实现实用优先（Utility-first）样式的核心组件。它不解析外部 CSS 文件，而是解析模板中的类名，并实时生成对应的 CSS 规则。

## 功能 (Features)

- **动态 CSS 生成**: 根据类名（如 `m-4`, `text-red`）生成对应的 CSS 声明。
- **高性能解析**: 基于 `nom` 库实现的高速类名解析引擎。
- **配置驱动**: 支持自定义颜色、间距等主题配置（计划中）。

## 维护指南 (Maintenance Guide)

### 核心逻辑
- `StyleEngine` ([lib.rs](file:///e:/模板引擎/project-hxo/compilers/hxo-parser-tailwind/src/lib.rs)): 维护已发现的类名并负责最终 CSS 的组装。
- `parse_classes`: 核心解析逻辑。

### 依赖项
- `nom`: 用于构建类名解析器。

### 测试
- 运行 `cargo test -p hxo-parser-tailwind`。
- 测试应包含各种边距、颜色、字体大小等原子类名的正确生成。
