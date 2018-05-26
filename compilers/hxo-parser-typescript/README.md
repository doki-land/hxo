# hxo-parser-typescript

TypeScript parser for HXO framework

## 目的 (Purpose)

`hxo-parser-typescript` 用于处理 HXO 文件中 `<script lang="ts">` 块的内容。它利用强大的 SWC 引擎对 TypeScript 代码进行解析、语法检查，并为后续的转译做准备。

## 功能 (Features)

- **TS 语法支持**: 支持最新的 TypeScript 语法特性。
- **AST 生成**: 生成标准的 SWC `Program` AST。
- **错误报告**: 将 SWC 的解析错误映射到 HXO 的错误系统中。

## 维护指南 (Maintenance Guide)

### 核心逻辑
- `TsParser` ([lib.rs](file:///e:/模板引擎/project-hxo/compilers/hxo-parser-typescript/src/lib.rs)): 封装 SWC 解析器的配置和调用流程。

### 依赖项
- `swc_ecma_parser`: 底层解析引擎。

### 测试
- 运行 `cargo test -p hxo-parser-typescript`。
- 测试包含各种 TS 特有的语法（接口、泛型、装饰器等）。
