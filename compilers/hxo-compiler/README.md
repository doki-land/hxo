# hxo-compiler

Compiler core for HXO framework

## 目的 (Purpose)

`hxo-compiler` 是 HXO 框架的核心编译引擎。它的主要任务是将 HXO 源代码转换成可执行的 JavaScript 和优化的 CSS。它负责协调解析、样式提取、模板编译和最终的代码生成流程。

## 功能 (Features)

- **集成编译流程**: 协调模板解析和样式处理。
- **自动化样式提取**: 从模板 class 属性中自动识别并生成 Tailwind-like 的 CSS。
- **统一 API**: 提供简洁的 `compile` 接口用于处理 HXO 源代码。
- **CSS 生成**: 通过集成的样式引擎生成高度优化的样式表。

## 维护指南 (Maintenance Guide)

### 核心逻辑
- `Compiler` 结构体 ([lib.rs](file:///e:/模板引擎/project-hxo/compilers/hxo-compiler/src/lib.rs)): 核心入口点，维护 `StyleEngine` 状态。
- `compile` 方法: 执行解析和样式收集。
- `collect_styles_from_nodes`: 递归遍历模板 AST 以提取样式类名。

### 依赖项
- `hxo-types`: 使用通用的位置和错误类型。
- `hxo-parser-template`: 用于解析模板结构。
- `hxo-parser-tailwind`: 用于处理和生成实用优先（Utility-first）的样式。

### 测试
- 运行 `cargo test -p hxo-compiler`。
- 测试用例覆盖了从模板中提取类名并生成对应 CSS 的完整链路。
