# hxo-bundler

Bundler for HXO framework

## 目的 (Purpose)

`hxo-bundler` 负责将编译后的 HXO 组件及其依赖项打包成可在浏览器或服务器运行的最终产物。它处理模块依赖关系图，并执行必要的代码合并与转换。

## 功能 (Features)

- **依赖分析**: 递归扫描组件的导入关系。
- **模块打包**: 生成兼容不同模块系统（如 ESM）的 bundle。
- **资产管理**: 处理非代码资源（如图片、字体）的引用。

## 维护指南 (Maintenance Guide)

### 核心逻辑
- 位于 [lib.rs](file:///e:/模板引擎/project-hxo/compilers/hxo-bundler/src/lib.rs) 的打包流程控制。

### 依赖项
- 未来可能集成 `rolldown` 或类似的 Rust 核心打包引擎。

### 测试
- 运行 `cargo test -p hxo-bundler`。
