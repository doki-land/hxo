# hxo-ssr

Server-Side Rendering support for HXO framework

## 目的 (Purpose)

`hxo-ssr` 提供了将 HXO 组件在服务器端渲染为 HTML 字符串的能力，旨在提升首屏加载速度和 SEO。

## 功能 (Features)

- **流式渲染**: 支持高效的 HTML 流式输出。
- **上下文管理**: 在渲染过程中处理全局状态和头部信息。
- **高性能输出**: 优化的字符串拼接和缓存机制。

## 维护指南 (Maintenance Guide)

### 核心逻辑
- 顶层渲染接口定义在 [lib.rs](file:///e:/模板引擎/project-hxo/compilers/hxo-ssr/src/lib.rs)。

### 依赖项
- `hxo-types`: 获取文件结构信息。

### 测试
- 运行 `cargo test -p hxo-ssr`。
