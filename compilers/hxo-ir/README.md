# hxo-ir

Intermediate Representation for HXO framework

## 目的 (Purpose)

`hxo-ir` 定义了 HXO 编译过程中的中间表示（Intermediate Representation）。它作为解析器输出和代码生成器输入之间的桥梁，提供了一个类型安全、易于操作的数据结构来表示整个 HXO 模块的语义信息。

## 功能 (Features)

- **模块化表示**: 使用 `IRModule` 统合元数据、脚本、模板和样式。
- **强类型 AST**: 为模板元素、属性和表达式定义了清晰的中间节点类型。
- **序列化支持**: 全面支持 `serde`，便于在编译器阶段之间进行持久化或跨进程传输。
- **扩展性**: 支持自定义块（Custom Blocks）的中间表示。

## 维护指南 (Maintenance Guide)

### 核心逻辑
- `IRModule` ([lib.rs](file:///e:/模板引擎/project-hxo/compilers/hxo-ir/src/lib.rs)): 顶层数据结构，包含模块的所有组成部分。
- `TemplateNodeIR`: 模板结构的中间表示枚举。
- `StyleIR`: 样式块及其元数据的封装。

### 依赖项
- `serde`: 用于数据结构的序列化和反序列化。
- `swc_ecma_ast`: 直接集成 SWC 的 AST 用于脚本部分的表示。

### 测试
- 运行 `cargo test -p hxo-ir`。
- 重点关注数据结构的正确性以及序列化/反序列化的兼容性。
