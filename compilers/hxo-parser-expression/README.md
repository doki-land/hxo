# hxo-parser-expression

HXO Template Expression language parser.

## 目的 (Purpose)

`hxo-parser-expression` 用于解析 HXO 模板中的表达式（如 `{{ expr }}`）以及简单的脚本块。它实现了一套类 JS/TS 的轻量级语法，专门为模板指令和响应式绑定设计。

## 功能 (Features)

- **轻量级 Pratt 解析器**: 手写的递归下降 + Pratt 解析器，支持 JS 常用表达式子集。
- **自动上下文支持**: 为后续编译器的自动 `ctx.` 注入提供 AST。
- **TSX 支持**: 在表达式中支持简单的类 TSX 标签语法 (`TseElement`)。
- **高性能**: 相比完整的 SWC/Acorn，解析开销极低。

## 维护指南 (Maintenance Guide)

### 核心逻辑
- `ExprParser` ([lib.rs](file:///e:/%E6%A8%A1%E6%9D%BF%E5%BC%95%E6%93%8E/project-hxo/compilers/hxo-parser-expression/src/lib.rs)): 实现 Pratt 解析算法的核心。

### 测试
- 运行 `cargo test -p hxo-parser-expression`。
- 测试用例位于 `tests/expr_test.rs`。
