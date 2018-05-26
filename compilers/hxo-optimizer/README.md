# hxo-optimizer

Optimizer for HXO framework

## 目的 (Purpose)

`hxo-optimizer` 在编译过程中对中间表示（IR）或生成的代码进行多轮优化，以减少产物体积并提升运行时性能。

## 功能 (Features)

- **常量折叠**: 在编译期计算静态表达式。
- **死代码删除**: 移除未使用的模板片段或脚本函数。
- **静态提升**: 将静态节点提升到渲染函数之外以减少重复开销。

## 维护指南 (Maintenance Guide)

### 核心逻辑
- 优化器转换规则集 ([lib.rs](file:///e:/模板引擎/project-hxo/compilers/hxo-optimizer/src/lib.rs))。

### 测试
- 运行 `cargo test -p hxo-optimizer`。
