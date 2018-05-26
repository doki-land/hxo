# hxo-hydrate

Hydration support for HXO framework

## 目的 (Purpose)

`hxo-hydrate` 提供客户端“注水”支持，用于在服务器端渲染（SSR）的 HTML 基础上恢复客户端的交互能力。

## 功能 (Features)

- **节点关联**: 实现服务器生成 DOM 与客户端 VDOM 的高效对比和关联。
- **事件恢复**: 在注水过程中重新绑定事件监听器。

## 维护指南 (Maintenance Guide)

### 核心逻辑
- 参见 [lib.rs](file:///e:/模板引擎/project-hxo/compilers/hxo-hydrate/src/lib.rs)。

### 测试
- 运行 `cargo test -p hxo-hydrate`。
