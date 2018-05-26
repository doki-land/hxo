# hxo-parser-json

JSON parser for HXO framework

## 目的 (Purpose)

`hxo-parser-json` 用于解析 HXO 文件中 `<metadata lang="json">` 块的内容。它主要用于处理组件的静态配置信息。

## 功能 (Features)

- **强类型解析**: 将 JSON 直接反序列化为 HXO 的配置结构体。
- **错误映射**: 提供友好的 JSON 语法错误提示。

## 维护指南 (Maintenance Guide)

### 核心逻辑
- 调用 `serde_json` 进行处理 ([lib.rs](file:///e:/模板引擎/project-hxo/compilers/hxo-parser-json/src/lib.rs))。

### 依赖项
- `serde_json`: 工业级 JSON 处理库。

### 测试
- 运行 `cargo test -p hxo-parser-json`。
