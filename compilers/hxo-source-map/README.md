# hxo-source-map

Source map generation for HXO framework

## 目的 (Purpose)

`hxo-source-map` 负责在编译过程中维护源代码与生成代码之间的映射关系，确保开发者可以在浏览器调试工具中直接查看并调试 HXO 源码。

## 功能 (Features)

- **映射生成**: 支持生成符合 Source Map V3 规范的映射文件。
- **跨块追踪**: 能够追踪从 SFC 不同块到最终 bundle 的位置变换。

## 维护指南 (Maintenance Guide)

### 核心逻辑
- 参见 [lib.rs](file:///e:/模板引擎/project-hxo/compilers/hxo-source-map/src/lib.rs)。

### 测试
- 运行 `cargo test -p hxo-source-map`。
