# hxo-parser-yaml

YAML parser for HXO framework

## 目的 (Purpose)

`hxo-parser-yaml` 用于解析 HXO 文件中 `<metadata lang="yaml">` 或 `<router>` 块的内容。YAML 是 HXO 默认的元数据配置格式。

## 功能 (Features)

- **配置解析**: 高效处理 YAML 格式的组件和路由配置。
- **兼容性**: 支持标准的 YAML 1.2 语法。

## 维护指南 (Maintenance Guide)

### 核心逻辑
- 使用 `serde_yaml` 进行反序列化 ([lib.rs](file:///e:/模板引擎/project-hxo/compilers/hxo-parser-yaml/src/lib.rs))。

### 依赖项
- `serde_yaml`: 基于 `unsafe-libyaml` 的高效封装。

### 测试
- 运行 `cargo test -p hxo-parser-yaml`。
